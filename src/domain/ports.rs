use async_trait::async_trait;

use super::model::{MonitoredAccount, TokenBalance};

#[async_trait]
pub trait BalanceProvider: Send + Sync {
    async fn fetch_balances(&self, account: &MonitoredAccount)
    -> anyhow::Result<Vec<TokenBalance>>;
    fn supports_network(&self, network: &str) -> bool;
}

#[async_trait]
pub trait Notifier: Send + Sync {
    async fn send_alert(&self, balance: &TokenBalance) -> anyhow::Result<()>;
}

pub trait MetricsRecorder: Send + Sync {
    fn record_balance(&self, balance: &TokenBalance);
}
