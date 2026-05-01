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

    /// Called once at the start of a check cycle. Implementations may use it
    /// to reset per-cycle bookkeeping (e.g. stale-series tracking).
    fn begin_cycle(&self) {}

    /// Called once after a check cycle finishes. Implementations may use it
    /// to prune label series that were not observed during the cycle.
    fn end_cycle(&self) {}
}
