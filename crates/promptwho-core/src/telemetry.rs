use crate::PromptwhoConfig;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing(_config: &PromptwhoConfig) {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .or_else(|_| tracing_subscriber::EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().compact())
        // .with(ErrorLayer::default())
        // .with_span_events(FmtSpan::CLOSE)
        // .with_target(true)
        .init();
}
