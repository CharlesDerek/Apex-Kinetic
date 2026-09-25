//! RTSP 1.0 over TCP with interleaved RTP. The downstream mTLS socket carries
//! each validated interleaved packet unchanged; it is a framed packet sink.
use anyhow::{bail, Context, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

const MAX_HEADER: usize = 16 * 1024;
const MAX_SDP: usize = 64 * 1024;
const MAX_PACKET: usize = 64 * 1024;

pub struct Session {
    stream: TcpStream,
    pending: Vec<u8>,
}

/// One packet is validated before a bounded downstream write. The caller
/// records readiness only after this returns successfully.
pub enum ForwardError {
    Camera(anyhow::Error),
    Downstream(anyhow::Error),
}

pub async fn forward_one<W: tokio::io::AsyncWrite + Unpin>(
    session: &mut Session,
    sink: &mut W,
) -> std::result::Result<(), ForwardError> {
    let packet = session.next_packet().await.map_err(ForwardError::Camera)?;
    timeout(Duration::from_secs(2), sink.write_all(&packet))
        .await
        .map_err(|_| ForwardError::Downstream(anyhow::anyhow!("downstream_backpressure")))?
        .map_err(|_| ForwardError::Downstream(anyhow::anyhow!("downstream_unavailable")))?;
    Ok(())
}

impl Session {
    pub async fn connect(url: &str) -> Result<Self> {
        let (host, port, path) = parse_url(url)?;
        let stream = timeout(
            Duration::from_secs(5),
            TcpStream::connect((host.as_str(), port)),
        )
        .await
        .context("camera_connect_timeout")?
        .context("camera_unreachable")?;
        let mut session = Self {
            stream,
            pending: Vec::new(),
        };
        let uri = format!("rtsp://{host}:{port}{path}");
        let describe = session
            .request(
                "DESCRIBE",
                &uri,
                1,
                &["Accept: application/sdp".to_string()],
            )
            .await?;
        let sdp = String::from_utf8(describe).context("invalid_sdp")?;
        let track = sdp
            .lines()
            .find_map(|line| line.strip_prefix("a=control:"))
            .filter(|v| !v.is_empty())
            .context("sdp_missing_track")?;
        let track_uri = if track.starts_with("rtsp://") {
            track.to_string()
        } else if track.starts_with('/') {
            format!("rtsp://{host}:{port}{track}")
        } else {
            format!("{uri}/{track}")
        };
        let (_, headers) = session
            .request_with_headers(
                "SETUP",
                &track_uri,
                2,
                &["Transport: RTP/AVP/TCP;unicast;interleaved=0-1".to_string()],
            )
            .await?;
        if !headers.lines().any(|line| {
            line.split_once(':').is_some_and(|(name, value)| {
                name.eq_ignore_ascii_case("transport")
                    && value.trim().starts_with("RTP/AVP/TCP;")
                    && value.contains("interleaved=0-1")
            })
        }) {
            bail!("rtsp_transport_mismatch");
        }
        let session_id = headers
            .lines()
            .find_map(|line| {
                line.strip_prefix("Session:")
                    .or_else(|| line.strip_prefix("session:"))
            })
            .and_then(|v| v.trim().split(';').next())
            .filter(|v| !v.is_empty())
            .context("rtsp_missing_session")?;
        if !session_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        {
            bail!("invalid_rtsp_session");
        }
        session
            .request("PLAY", &uri, 3, &[format!("Session: {session_id}")])
            .await?;
        Ok(session)
    }

    async fn request(
        &mut self,
        method: &str,
        uri: &str,
        cseq: u32,
        extra: &[String],
    ) -> Result<Vec<u8>> {
        self.request_with_headers(method, uri, cseq, extra)
            .await
            .map(|v| v.0)
    }

    async fn request_with_headers(
        &mut self,
        method: &str,
        uri: &str,
        cseq: u32,
        extra: &[String],
    ) -> Result<(Vec<u8>, String)> {
        let mut request =
            format!("{method} {uri} RTSP/1.0\r\nCSeq: {cseq}\r\nUser-Agent: apex-kinetic/1\r\n");
        for header in extra {
            request.push_str(header);
            request.push_str("\r\n");
        }
        request.push_str("\r\n");
        timeout(
            Duration::from_secs(5),
            self.stream.write_all(request.as_bytes()),
        )
        .await
        .context("rtsp_write_timeout")?
        .context("rtsp_write_failed")?;
        let header_end = loop {
            if let Some(pos) = self.pending.windows(4).position(|w| w == b"\r\n\r\n") {
                break pos + 4;
            }
            if self.pending.len() >= MAX_HEADER {
                bail!("rtsp_header_too_large");
            }
            self.read_more().await?;
        };
        let headers = String::from_utf8(self.pending[..header_end].to_vec())
            .context("invalid_rtsp_header")?;
        let mut lines = headers.lines();
        let status = lines.next().context("missing_rtsp_status")?;
        let code = status
            .split_whitespace()
            .nth(1)
            .context("invalid_rtsp_status")?;
        if !status.starts_with("RTSP/1.0 ") {
            bail!("invalid_rtsp_status");
        }
        if code == "401" || code == "403" {
            bail!("camera_authentication_failed");
        }
        if code != "200" {
            bail!("rtsp_negotiation_failed");
        }
        let response_cseq = lines.clone().find_map(|line| {
            line.split_once(':')
                .filter(|(k, _)| k.eq_ignore_ascii_case("cseq"))
                .map(|(_, v)| v.trim().to_owned())
        });
        if response_cseq.as_deref() != Some(cseq.to_string().as_str()) {
            bail!("rtsp_cseq_mismatch");
        }
        let content_len = lines
            .find_map(|line| {
                line.split_once(':')
                    .filter(|(k, _)| k.eq_ignore_ascii_case("content-length"))
                    .and_then(|(_, v)| v.trim().parse::<usize>().ok())
            })
            .unwrap_or(0);
        if content_len > MAX_SDP {
            bail!("rtsp_body_too_large");
        }
        while self.pending.len() < header_end + content_len {
            self.read_more().await?;
        }
        let body = self.pending[header_end..header_end + content_len].to_vec();
        self.pending.drain(..header_end + content_len);
        Ok((body, headers))
    }

    async fn read_more(&mut self) -> Result<()> {
        let mut bytes = [0u8; 4096];
        let n = timeout(Duration::from_secs(5), self.stream.read(&mut bytes))
            .await
            .context("camera_media_timeout")?
            .context("camera_read_failed")?;
        if n == 0 {
            bail!("camera_disconnected");
        }
        self.pending.extend_from_slice(&bytes[..n]);
        if self.pending.len() > MAX_SDP + MAX_HEADER + MAX_PACKET {
            bail!("camera_buffer_limit");
        }
        Ok(())
    }

    pub async fn next_packet(&mut self) -> Result<Vec<u8>> {
        while self.pending.len() < 4 {
            self.read_more().await?;
        }
        if self.pending[0] != b'$' {
            bail!("malformed_interleaved_media");
        }
        let len = u16::from_be_bytes([self.pending[2], self.pending[3]]) as usize;
        if !(12..=MAX_PACKET).contains(&len) {
            bail!("invalid_rtp_length");
        }
        while self.pending.len() < 4 + len {
            self.read_more().await?;
        }
        let packet = self.pending.drain(..4 + len).collect::<Vec<_>>();
        if packet[1] > 1 || packet[4] >> 6 != 2 {
            bail!("invalid_rtp_packet");
        }
        Ok(packet)
    }
}

fn parse_url(url: &str) -> Result<(String, u16, String)> {
    let rest = url.strip_prefix("rtsp://").context("invalid_rtsp_url")?;
    let (authority, path) = rest.split_once('/').context("rtsp_path_required")?;
    if authority.contains('@') {
        bail!("rtsp_credentials_unsupported");
    }
    let (host, port) = match authority.split_once(':') {
        Some((host, port)) => (host, port.parse::<u16>().context("invalid_rtsp_port")?),
        None => (authority, 554),
    };
    if host.is_empty()
        || !host
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
        || path.is_empty()
        || path.contains(['\r', '\n'])
    {
        bail!("invalid_rtsp_url");
    }
    Ok((host.to_string(), port, format!("/{path}")))
}
