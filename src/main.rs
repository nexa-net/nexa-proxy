use std::sync::Arc;

use clap::Parser;
use nexa_proxy::{config, metrics, proxy};
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "nexa-proxy", about = "NexaNet built-in reverse proxy", version)]
struct Cli {
    #[arg(long, default_value = "/var/lib/nexa/proxy.json")]
    config: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    info!("starting nexa-proxy");

    let config = config::ProxyConfig::load(std::path::Path::new(&cli.config))?;
    info!(http = %config.http_listen, "loaded proxy config with {} routes", config.routes.len());

    let prom = Arc::new(metrics::ProxyPrometheusMetrics::new());
    let state = Arc::new(proxy::ProxyState::from_config(&config, Some(prom)));

    proxy::run_http(&config.http_listen, state).await
}
