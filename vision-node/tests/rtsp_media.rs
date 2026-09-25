use apex_kinetic_vision_node::health::{HealthReporter, HealthStatus};
use apex_kinetic_vision_node::rtsp_media::{forward_one, ForwardError, Session};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

async fn camera_fixture(packet: Vec<u8>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        for (cseq, response) in [
            "Content-Type: application/sdp\r\nContent-Length: 20\r\n\r\na=control:trackID=0\n"
                .to_string(),
            "Session: fixture-1\r\nTransport: RTP/AVP/TCP;unicast;interleaved=0-1\r\n\r\n"
                .to_string(),
            "Session: fixture-1\r\n\r\n".to_string(),
        ]
        .iter()
        .enumerate()
        {
            let mut request = vec![0u8; 2048];
            let _ = stream.read(&mut request).await.unwrap();
            let reply = format!("RTSP/1.0 200 OK\r\nCSeq: {}\r\n{}", cseq + 1, response);
            stream.write_all(reply.as_bytes()).await.unwrap();
        }
        stream.write_all(&packet).await.unwrap();
    });
    format!("rtsp://127.0.0.1:{}/stream", address.port())
}

#[tokio::test]
async fn fixture_media_reaches_bounded_downstream_sink() {
    let mut packet = vec![b'$', 0, 0, 12];
    packet.extend_from_slice(&[0x80, 96, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1]);
    let url = camera_fixture(packet.clone()).await;
    let mut session = Session::connect(&url).await.unwrap();
    let (mut sink, mut downstream) = tokio::io::duplex(128);
    let mut health = HealthReporter::new("vision-node", "fixture", 1);
    health.record_starting("camera");
    health.record_starting("nvr");
    assert_eq!(health.event(1).status, HealthStatus::Starting);
    assert!(forward_one(&mut session, &mut sink).await.is_ok());
    health.record_success("camera", "validated RTP media recent");
    health.record_success("nvr", "media forwarded over mTLS");
    assert_eq!(health.event(2).status, HealthStatus::Ready);
    let mut forwarded = vec![0; packet.len()];
    downstream.read_exact(&mut forwarded).await.unwrap();
    assert_eq!(forwarded, packet);
}

#[tokio::test]
async fn malformed_media_is_rejected() {
    let url = camera_fixture(vec![b'$', 0, 0, 12, 0x40, 96, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1]).await;
    let mut session = Session::connect(&url).await.unwrap();
    let (mut sink, _) = tokio::io::duplex(128);
    assert!(matches!(
        forward_one(&mut session, &mut sink).await,
        Err(ForwardError::Camera(_))
    ));
}

#[tokio::test]
async fn source_loss_and_slow_downstream_fail_closed() {
    let mut packet = vec![b'$', 0, 0, 12];
    packet.extend_from_slice(&[0x80, 96, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1]);
    let url = camera_fixture(packet.clone()).await;
    let mut session = Session::connect(&url).await.unwrap();
    assert_eq!(session.next_packet().await.unwrap(), packet);
    assert!(session.next_packet().await.is_err());

    let url = camera_fixture(packet).await;
    let mut session = Session::connect(&url).await.unwrap();
    let (mut blocked, _reader) = tokio::io::duplex(1);
    assert!(matches!(
        forward_one(&mut session, &mut blocked).await,
        Err(ForwardError::Downstream(_))
    ));
}

#[tokio::test]
async fn credentials_are_refused_without_logging_url() {
    let error = match Session::connect("rtsp://secret:password@camera/stream").await {
        Ok(_) => panic!("accepted credentials"),
        Err(error) => error,
    };
    assert_eq!(error.to_string(), "rtsp_credentials_unsupported");
}
