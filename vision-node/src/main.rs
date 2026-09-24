use anyhow::Result;
use apex_kinetic_vision_node::health::{is_snapshot_ready, DEFAULT_HEALTH_STATE_PATH};
use apex_kinetic_vision_node::{run, VisionNodeConfig};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    if std::env::args().nth(1).as_deref() == Some("--health-check") {
        let path = std::env::var("VISION_HEALTH_STATE_PATH")
            .unwrap_or_else(|_| DEFAULT_HEALTH_STATE_PATH.to_string());
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis()
            .min(u64::MAX as u128) as u64;
        if is_snapshot_ready(Path::new(&path), now, 60_000)? {
            return Ok(());
        }
        anyhow::bail!("vision-node is not ready");
    }
    run(VisionNodeConfig::from_env()).await
}
