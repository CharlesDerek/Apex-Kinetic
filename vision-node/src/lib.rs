use anyhow::{Context, Result};
use log::info;
use std::{env, fs, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_rustls::{
    rustls::{Certificate, ClientConfig, PrivateKey, RootCertStore},
    TlsConnector,
};

pub mod rtsp_control;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VisionNodeConfig {
    pub source_url: String,
    pub target_host: String,
    pub target_port: String,
    pub cert_path: String,
    pub key_path: String,
    pub ca_path: String,
}

impl VisionNodeConfig {
    pub fn from_env() -> Self {
        Self {
            source_url: env::var("RTSP_SOURCE_URL")
                .unwrap_or_else(|_| "rtsp://edge-camera.local/stream".into()),
            target_host: env::var("NVR_TARGET_HOST").unwrap_or_else(|_| "annke-nvr.local".into()),
            target_port: env::var("NVR_TARGET_PORT").unwrap_or_else(|_| "554".into()),
            cert_path: env::var("MTLS_CERT_PATH")
                .unwrap_or_else(|_| "/etc/certs/client.crt".into()),
            key_path: env::var("MTLS_KEY_PATH").unwrap_or_else(|_| "/etc/certs/client.key".into()),
            ca_path: env::var("MTLS_CA_PATH").unwrap_or_else(|_| "/etc/certs/ca.crt".into()),
        }
    }
}

pub async fn run(config: VisionNodeConfig) -> Result<()> {
    info!("Starting vision-node ingress engine");
    info!("RTSP source: {}", config.source_url);
    info!("Target NVR: {}:{}", config.target_host, config.target_port);

    let connector = build_tls_connector(&config.cert_path, &config.key_path, &config.ca_path)
        .context("Failed to build TLS connector")?;

    let rtsp_stream = connect_to_nvr(&connector, &config.target_host, &config.target_port)
        .await
        .context("Failed to connect to NVR")?;

    bridge_rtsp_to_nvr(rtsp_stream, config.source_url).await
}

pub async fn connect_to_nvr(
    connector: &TlsConnector,
    host: &str,
    port: &str,
) -> Result<tokio_rustls::client::TlsStream<TcpStream>> {
    let tcp = TcpStream::connect(format!("{}:{}", host, port))
        .await
        .context("Failed to open TCP connection to NVR")?;
    let server_name =
        rustls::ServerName::try_from(host).context("Invalid NVR host name for TLS")?;
    let tls_stream = connector.connect(server_name, tcp).await?;
    info!("Established mTLS session to NVR target");
    Ok(tls_stream)
}

pub async fn bridge_rtsp_to_nvr(
    mut tls_stream: tokio_rustls::client::TlsStream<TcpStream>,
    source_url: String,
) -> Result<()> {
    info!("Starting RTSP ingestion loop for source {}", source_url);

    let mut buffer = [0u8; 1024];
    loop {
        let sample = generate_dummy_video_payload(&source_url);
        tls_stream
            .write_all(&sample)
            .await
            .context("Failed to send payload to NVR")?;
        let n = tls_stream.read(&mut buffer).await.unwrap_or(0);
        if n > 0 {
            info!("Received {} bytes from target NVR", n);
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

pub fn generate_dummy_video_payload(source_url: &str) -> Vec<u8> {
    format!("RTSP_FRAME from {}\n", source_url).into_bytes()
}

pub fn build_tls_connector(cert_path: &str, key_path: &str, ca_path: &str) -> Result<TlsConnector> {
    let mut root_store = RootCertStore::empty();
    let ca_cert = fs::read(ca_path).context("Unable to read CA certificate")?;
    let ca_certs =
        rustls_pemfile::certs(&mut &*ca_cert).context("Unable to parse CA certificate")?;
    for cert in ca_certs {
        root_store.add(&Certificate(cert))?;
    }

    let cert_file = fs::read(cert_path).context("Unable to read client certificate")?;
    let key_file = fs::read(key_path).context("Unable to read client private key")?;
    let certs = rustls_pemfile::certs(&mut &*cert_file)
        .context("Unable to parse client certificate")?
        .into_iter()
        .map(Certificate)
        .collect();
    let key = rustls_pemfile::pkcs8_private_keys(&mut &*key_file)
        .context("Unable to parse client private key")?
        .into_iter()
        .next()
        .map(PrivateKey)
        .context("Client private key not found")?;

    let mut config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_single_cert(certs, key)?;

    config.alpn_protocols.push(b"h2".to_vec());
    Ok(TlsConnector::from(Arc::new(config)))
}
