mod config;
mod domain;
mod infra;
mod tokens;

use std::sync::Arc;

use alloy::providers::ProviderBuilder;
use tokio::time::Duration;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use domain::model::{MonitoredAccount, NetworkType};
use domain::ports::BalanceProvider;
use domain::service::BalanceMonitorService;
use infra::evm::EvmBalanceProvider;
use infra::metrics::{PrometheusRecorder, serve_metrics};
use infra::solana::SolanaBalanceProvider;
use infra::telegram::TelegramNotifier;
use infra::tron_http::TronHttpProvider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("balance_control=info")),
        )
        .init();

    let cfg = config::load("config.yaml")?;
    info!(
        networks = cfg.networks.len(),
        accounts = cfg.accounts.len(),
        "Configuration loaded"
    );

    let mut providers: Vec<Arc<dyn BalanceProvider>> = Vec::new();

    for (name, net_cfg) in &cfg.networks {
        match NetworkType::from_name(name) {
            NetworkType::Evm => {
                let provider = ProviderBuilder::new().connect(&net_cfg.rpc_url).await?;
                let p = EvmBalanceProvider::new(name.clone(), provider);
                providers.push(Arc::new(p));
            }
            NetworkType::Tron => {
                let p = TronHttpProvider::new(name.clone(), &net_cfg.rpc_url);
                providers.push(Arc::new(p));
            }
            NetworkType::Solana => {
                let p = SolanaBalanceProvider::new(name.clone(), &net_cfg.rpc_url);
                providers.push(Arc::new(p));
            }
        }
        info!(network = name, "Provider initialized");
    }

    let notifier = Arc::new(TelegramNotifier::new(
        &cfg.settings.telegram.token,
        cfg.settings.telegram.chat_id,
    ));

    let metrics = Arc::new(PrometheusRecorder::new()?);

    let accounts: Vec<MonitoredAccount> = cfg.accounts.iter().map(MonitoredAccount::from).collect();

    let metrics_port = cfg.settings.prometheus_port;
    let metrics_clone = metrics.clone();
    tokio::spawn(async move {
        if let Err(e) = serve_metrics(metrics_clone, metrics_port).await {
            error!(error = %e, "Metrics server failed");
        }
    });

    let service = BalanceMonitorService::new(
        providers,
        notifier,
        metrics,
        accounts,
        Duration::from_secs(cfg.settings.check_interval_secs),
    );

    info!("Balance monitor started");
    service.run().await;
    Ok(())
}
