use anyhow::Result;
use promptwho_core::PromptwhoConfig;

use promptwho_core::telemetry::init_tracing;
use promptwho_server::run;

#[tokio::main]
async fn main() -> Result<()> {
    let config = PromptwhoConfig::load(None);

    init_tracing(&config);

    let _ = run(&config).await;

    Ok(())
}
