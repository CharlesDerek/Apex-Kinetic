use anyhow::{anyhow, Context, Result};
use log::{info, warn};
use std::{
    env, fs,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_rustls::{
    rustls::{Certificate, ClientConfig, PrivateKey, RootCertStore},
    TlsConnector,
};

pub mod health;
pub mod rtsp_control;
use health::{HealthReporter, KafkaHealthPublisher, DEFAULT_HEALTH_STATE_PATH};

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
    info!("RTSP source configured");
    info!("Target NVR: {}:{}", config.target_host, config.target_port);

    let bootstrap_servers =
        env::var("KAFKA_BOOTSTRAP_SERVERS").unwrap_or_else(|_| "localhost:9092".to_string());
    let publisher = KafkaHealthPublisher::new(&bootstrap_servers)?;
    let instance_id = env::var("HOSTNAME").unwrap_or_else(|_| "vision-node".to_string());
    let mut health = HealthReporter::new("vision-node", instance_id, 2);
    health.record_starting("certificate");
    health.record_starting("nvr");
    publish_health_best_effort(&publisher, &mut health).await;

    let connector = match build_tls_connector(&config.cert_path, &config.key_path, &config.ca_path)
    {
        Ok(connector) => connector,
        Err(error) => {
            health.record_unhealthy("certificate", "certificate configuration failed");
            publish_health_best_effort(&publisher, &mut health).await;
            return Err(error).context("Failed to build TLS connector");
        }
    };
    health.record_success("certificate", "loaded");
    publish_health_best_effort(&publisher, &mut health).await;

    let rtsp_stream =
        match connect_to_nvr(&connector, &config.target_host, &config.target_port).await {
            Ok(stream) => stream,
            Err(error) => {
                health.record_unhealthy("nvr", "mTLS connection failed");
                publish_health_best_effort(&publisher, &mut health).await;
                return Err(error).context("Failed to connect to NVR");
            }
        };
    health.record_success("nvr", "mTLS connected");
    publish_health_best_effort(&publisher, &mut health).await;

    bridge_rtsp_to_nvr_with_health(
        rtsp_stream,
        config.source_url,
        Some((&publisher, &mut health)),
    )
    .await
}

pub async fn connect_to_nvr(
    connector: &TlsConnector,
    host: &str,
    port: &str,
) -> Result<tokio_rustls::client::TlsStream<TcpStream>> {
    let tcp = tokio::time::timeout(
        Duration::from_secs(10),
        TcpStream::connect(format!("{}:{}", host, port)),
    )
    .await
    .context("Timed out opening TCP connection to NVR")?
    .context("Failed to open TCP connection to NVR")?;
    let server_name =
        rustls::ServerName::try_from(host).context("Invalid NVR host name for TLS")?;
    let tls_stream =
        tokio::time::timeout(Duration::from_secs(10), connector.connect(server_name, tcp))
            .await
            .context("Timed out completing NVR mTLS handshake")??;
    info!("Established mTLS session to NVR target");
    Ok(tls_stream)
}

pub async fn bridge_rtsp_to_nvr(
    tls_stream: tokio_rustls::client::TlsStream<TcpStream>,
    source_url: String,
) -> Result<()> {
    bridge_rtsp_to_nvr_with_health(tls_stream, source_url, None).await
}

async fn bridge_rtsp_to_nvr_with_health(
    mut tls_stream: tokio_rustls::client::TlsStream<TcpStream>,
    source_url: String,
    mut health: Option<(&KafkaHealthPublisher, &mut HealthReporter)>,
) -> Result<()> {
    info!("Starting RTSP ingestion loop");

    let mut buffer = [0u8; 1024];
    let mut last_health = tokio::time::Instant::now();
    loop {
        let sample = generate_dummy_video_payload(&source_url);
        let write_result =
            tokio::time::timeout(Duration::from_secs(5), tls_stream.write_all(&sample))
                .await
                .context("NVR stream write timed out")
                .and_then(|result| result.context("NVR stream write failed"));
        if let Err(error) = write_result {
            if let Some((publisher, reporter)) = health.as_mut() {
                reporter.record_unhealthy("nvr", "stream write failed");
                publish_health_best_effort(publisher, reporter).await;
            }
            return Err(error).context("Failed to send payload to NVR");
        }
        match tokio::time::timeout(Duration::from_millis(500), tls_stream.read(&mut buffer)).await {
            Ok(Ok(0)) => {
                if let Some((publisher, reporter)) = health.as_mut() {
                    reporter.record_unhealthy("nvr", "connection closed");
                    publish_health_best_effort(publisher, reporter).await;
                }
                return Err(anyhow!("NVR connection closed"));
            }
            Ok(Ok(n)) => info!("Received {} bytes from target NVR", n),
            Ok(Err(error)) => {
                if let Some((publisher, reporter)) = health.as_mut() {
                    reporter.record_unhealthy("nvr", "stream read failed");
                    publish_health_best_effort(publisher, reporter).await;
                }
                return Err(error).context("Failed to read from NVR");
            }
            Err(_) => {}
        }
        if last_health.elapsed() >= Duration::from_secs(30) {
            if let Some((publisher, reporter)) = health.as_mut() {
                reporter.record_success("nvr", "mTLS stream active");
                publish_health_best_effort(publisher, reporter).await;
            }
            last_health = tokio::time::Instant::now();
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn publish_health_best_effort(
    publisher: &KafkaHealthPublisher,
    reporter: &mut HealthReporter,
) {
    let observed_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u64::MAX as u128) as u64)
        .unwrap_or_default();
    let event = reporter.event(observed_at);
    let snapshot_path = env::var("VISION_HEALTH_STATE_PATH")
        .unwrap_or_else(|_| DEFAULT_HEALTH_STATE_PATH.to_string());
    if let Err(error) = health::write_health_snapshot(std::path::Path::new(&snapshot_path), &event)
    {
        warn!("health snapshot write failed: {error}");
    }
    if let Err(error) = publisher.publish(&event).await {
        warn!("health event publish failed: {error}");
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
