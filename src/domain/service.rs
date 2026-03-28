use std::sync::Arc;

use futures::future::join_all;
use tokio::time::{self, Duration};
use tracing::{error, info, warn};

use super::model::MonitoredAccount;
use super::ports::{BalanceProvider, MetricsRecorder, Notifier};

pub struct BalanceMonitorService {
    providers: Vec<Arc<dyn BalanceProvider>>,
    notifier: Arc<dyn Notifier>,
    metrics: Arc<dyn MetricsRecorder>,
    accounts: Vec<MonitoredAccount>,
    interval: Duration,
}

impl BalanceMonitorService {
    pub fn new(
        providers: Vec<Arc<dyn BalanceProvider>>,
        notifier: Arc<dyn Notifier>,
        metrics: Arc<dyn MetricsRecorder>,
        accounts: Vec<MonitoredAccount>,
        interval: Duration,
    ) -> Self {
        Self {
            providers,
            notifier,
            metrics,
            accounts,
            interval,
        }
    }

    pub async fn run(&self) {
        let mut tick = time::interval(self.interval);
        loop {
            tick.tick().await;
            info!("Starting balance check cycle");
            self.check_cycle().await;
            info!("Balance check cycle complete");
        }
    }

    pub async fn check_cycle(&self) {
        let futs: Vec<_> = self
            .accounts
            .iter()
            .map(|acc| self.check_account(acc))
            .collect();
        join_all(futs).await;
    }

    pub async fn check_account(&self, account: &MonitoredAccount) {
        let Some(provider) = self.provider_for(&account.network) else {
            warn!(network = %account.network, alias = %account.alias, "No provider found");
            return;
        };

        match provider.fetch_balances(account).await {
            Ok(balances) => {
                for bal in &balances {
                    self.metrics.record_balance(bal);
                    if bal.is_below_threshold() {
                        warn!(%bal, "Balance below threshold");
                        if let Err(e) = self.notifier.send_alert(bal).await {
                            error!(error = %e, "Failed to send alert");
                        }
                    }
                }
            }
            Err(e) => {
                error!(
                    alias = %account.alias,
                    network = %account.network,
                    error = %e,
                    "Failed to fetch balances"
                );
            }
        }
    }

    fn provider_for(&self, network: &str) -> Option<&Arc<dyn BalanceProvider>> {
        self.providers.iter().find(|p| p.supports_network(network))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::{TokenBalance, TokenId};
    use async_trait::async_trait;
    use rust_decimal_macros::dec;
    use std::sync::Mutex;

    struct MockBalanceProvider {
        network: String,
        balances: Vec<TokenBalance>,
    }

    impl MockBalanceProvider {
        fn new(network: &str, balances: Vec<TokenBalance>) -> Self {
            Self {
                network: network.to_string(),
                balances,
            }
        }
    }

    #[async_trait]
    impl BalanceProvider for MockBalanceProvider {
        async fn fetch_balances(
            &self,
            _account: &MonitoredAccount,
        ) -> anyhow::Result<Vec<TokenBalance>> {
            Ok(self.balances.clone())
        }

        fn supports_network(&self, network: &str) -> bool {
            self.network == network
        }
    }

    struct MockNotifier {
        alerts: Arc<Mutex<Vec<TokenBalance>>>,
    }

    impl MockNotifier {
        fn new() -> Self {
            Self {
                alerts: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn get_alerts(&self) -> Vec<TokenBalance> {
            self.alerts.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl Notifier for MockNotifier {
        async fn send_alert(&self, balance: &TokenBalance) -> anyhow::Result<()> {
            self.alerts.lock().unwrap().push(balance.clone());
            Ok(())
        }
    }

    struct MockMetricsRecorder {
        recorded: Arc<Mutex<Vec<TokenBalance>>>,
    }

    impl MockMetricsRecorder {
        fn new() -> Self {
            Self {
                recorded: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn get_recorded(&self) -> Vec<TokenBalance> {
            self.recorded.lock().unwrap().clone()
        }
    }

    impl MetricsRecorder for MockMetricsRecorder {
        fn record_balance(&self, balance: &TokenBalance) {
            self.recorded.lock().unwrap().push(balance.clone());
        }
    }

    #[tokio::test]
    async fn test_check_account_below_threshold() {
        let balance = TokenBalance {
            account_address: "0x123".to_string(),
            account_alias: "TestWallet".to_string(),
            network: "ethereum".to_string(),
            token: "USDT".to_string(),
            balance: dec!(50.0),
            threshold: dec!(100.0),
        };

        let provider = Arc::new(MockBalanceProvider::new("ethereum", vec![balance.clone()]));
        let notifier = Arc::new(MockNotifier::new());
        let metrics = Arc::new(MockMetricsRecorder::new());

        let account = MonitoredAccount {
            address: "0x123".to_string(),
            alias: "TestWallet".to_string(),
            network: "ethereum".to_string(),
            threshold: dec!(100.0),
            tokens: vec![TokenId::Native],
        };

        let service = BalanceMonitorService::new(
            vec![provider],
            notifier.clone(),
            metrics.clone(),
            vec![account.clone()],
            Duration::from_secs(60),
        );

        service.check_account(&account).await;

        let alerts = notifier.get_alerts();
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].balance, dec!(50.0));

        let recorded = metrics.get_recorded();
        assert_eq!(recorded.len(), 1);
    }

    #[tokio::test]
    async fn test_check_account_above_threshold() {
        let balance = TokenBalance {
            account_address: "0x123".to_string(),
            account_alias: "TestWallet".to_string(),
            network: "ethereum".to_string(),
            token: "USDT".to_string(),
            balance: dec!(150.0),
            threshold: dec!(100.0),
        };

        let provider = Arc::new(MockBalanceProvider::new("ethereum", vec![balance.clone()]));
        let notifier = Arc::new(MockNotifier::new());
        let metrics = Arc::new(MockMetricsRecorder::new());

        let account = MonitoredAccount {
            address: "0x123".to_string(),
            alias: "TestWallet".to_string(),
            network: "ethereum".to_string(),
            threshold: dec!(100.0),
            tokens: vec![TokenId::Native],
        };

        let service = BalanceMonitorService::new(
            vec![provider],
            notifier.clone(),
            metrics.clone(),
            vec![account.clone()],
            Duration::from_secs(60),
        );

        service.check_account(&account).await;

        let alerts = notifier.get_alerts();
        assert_eq!(alerts.len(), 0);

        let recorded = metrics.get_recorded();
        assert_eq!(recorded.len(), 1);
    }

    #[tokio::test]
    async fn test_provider_for_network() {
        let provider1 = Arc::new(MockBalanceProvider::new("ethereum", vec![]));
        let provider2 = Arc::new(MockBalanceProvider::new("bsc", vec![]));

        let service = BalanceMonitorService::new(
            vec![provider1, provider2],
            Arc::new(MockNotifier::new()),
            Arc::new(MockMetricsRecorder::new()),
            vec![],
            Duration::from_secs(60),
        );

        assert!(service.provider_for("ethereum").is_some());
        assert!(service.provider_for("bsc").is_some());
        assert!(service.provider_for("polygon").is_none());
    }

    #[tokio::test]
    async fn test_check_cycle_multiple_accounts() {
        let balance1 = TokenBalance {
            account_address: "0x123".to_string(),
            account_alias: "Wallet1".to_string(),
            network: "ethereum".to_string(),
            token: "USDT".to_string(),
            balance: dec!(50.0),
            threshold: dec!(100.0),
        };

        let balance2 = TokenBalance {
            account_address: "0x456".to_string(),
            account_alias: "Wallet2".to_string(),
            network: "bsc".to_string(),
            token: "USDC".to_string(),
            balance: dec!(200.0),
            threshold: dec!(100.0),
        };

        let provider1 = Arc::new(MockBalanceProvider::new("ethereum", vec![balance1]));
        let provider2 = Arc::new(MockBalanceProvider::new("bsc", vec![balance2]));
        let notifier = Arc::new(MockNotifier::new());
        let metrics = Arc::new(MockMetricsRecorder::new());

        let accounts = vec![
            MonitoredAccount {
                address: "0x123".to_string(),
                alias: "Wallet1".to_string(),
                network: "ethereum".to_string(),
                threshold: dec!(100.0),
                tokens: vec![TokenId::Native],
            },
            MonitoredAccount {
                address: "0x456".to_string(),
                alias: "Wallet2".to_string(),
                network: "bsc".to_string(),
                threshold: dec!(100.0),
                tokens: vec![TokenId::Native],
            },
        ];

        let service = BalanceMonitorService::new(
            vec![provider1, provider2],
            notifier.clone(),
            metrics.clone(),
            accounts,
            Duration::from_secs(60),
        );

        service.check_cycle().await;

        let alerts = notifier.get_alerts();
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].account_alias, "Wallet1");

        let recorded = metrics.get_recorded();
        assert_eq!(recorded.len(), 2);
    }
}
