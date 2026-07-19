use anyhow::Result;
use apex_kinetic_vision_node::{run, VisionNodeConfig};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    run(VisionNodeConfig::from_env()).await
}
