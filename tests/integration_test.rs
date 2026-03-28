use balance_control::domain::model::{MonitoredAccount, TokenBalance, TokenId};
use balance_control::domain::ports::{BalanceProvider, MetricsRecorder, Notifier};
use balance_control::domain::service::BalanceMonitorService;
use async_trait::async_trait;
use rust_decimal_macros::dec;
use std::sync::{Arc, Mutex};
use tokio::time::Duration;

struct TestBalanceProvider {
    network: String,
    balances: Vec<TokenBalance>,
    call_count: Arc<Mutex<usize>>,
}

impl TestBalanceProvider {
    fn new(network: &str, balances: Vec<TokenBalance>) -> Self {
        Self {
            network: network.to_string(),
            balances,
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    fn get_call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }
}

#[async_trait]
impl BalanceProvider for TestBalanceProvider {
    async fn fetch_balances(&self, _account: &MonitoredAccount) -> anyhow::Result<Vec<TokenBalance>> {
        *self.call_count.lock().unwrap() += 1;
        Ok(self.balances.clone())
    }

    fn supports_network(&self, network: &str) -> bool {
        self.network == network
    }
}

struct TestNotifier {
    alerts: Arc<Mutex<Vec<TokenBalance>>>,
}

impl TestNotifier {
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
impl Notifier for TestNotifier {
    async fn send_alert(&self, balance: &TokenBalance) -> anyhow::Result<()> {
        self.alerts.lock().unwrap().push(balance.clone());
        Ok(())
    }
}

struct TestMetricsRecorder {
    recorded: Arc<Mutex<Vec<TokenBalance>>>,
}

impl TestMetricsRecorder {
    fn new() -> Self {
        Self {
            recorded: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_recorded(&self) -> Vec<TokenBalance> {
        self.recorded.lock().unwrap().clone()
    }
}

impl MetricsRecorder for TestMetricsRecorder {
    fn record_balance(&self, balance: &TokenBalance) {
        self.recorded.lock().unwrap().push(balance.clone());
    }
}

#[tokio::test]
async fn test_multi_network_monitoring() {
    let eth_balance = TokenBalance {
        account_address: "0x123".to_string(),
        account_alias: "EthWallet".to_string(),
        network: "ethereum".to_string(),
        token: "ETH".to_string(),
        balance: dec!(0.5),
        threshold: dec!(1.0),
    };

    let bsc_balance = TokenBalance {
        account_address: "0x456".to_string(),
        account_alias: "BscWallet".to_string(),
        network: "bsc".to_string(),
        token: "BNB".to_string(),
        balance: dec!(5.0),
        threshold: dec!(2.0),
    };

    let eth_provider = Arc::new(TestBalanceProvider::new("ethereum", vec![eth_balance]));
    let bsc_provider = Arc::new(TestBalanceProvider::new("bsc", vec![bsc_balance]));
    let notifier = Arc::new(TestNotifier::new());
    let metrics = Arc::new(TestMetricsRecorder::new());

    let accounts = vec![
        MonitoredAccount {
            address: "0x123".to_string(),
            alias: "EthWallet".to_string(),
            network: "ethereum".to_string(),
            threshold: dec!(1.0),
            tokens: vec![TokenId::Native],
        },
        MonitoredAccount {
            address: "0x456".to_string(),
            alias: "BscWallet".to_string(),
            network: "bsc".to_string(),
            threshold: dec!(2.0),
            tokens: vec![TokenId::Native],
        },
    ];

    let service = BalanceMonitorService::new(
        vec![eth_provider.clone(), bsc_provider.clone()],
        notifier.clone(),
        metrics.clone(),
        accounts,
        Duration::from_secs(60),
    );

    // Simulate one check cycle
    tokio::time::timeout(Duration::from_secs(1), async {
        service.check_cycle().await;
    })
    .await
    .unwrap();

    // Verify alerts (only ETH should alert)
    let alerts = notifier.get_alerts();
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].network, "ethereum");

    // Verify metrics recorded for both
    let recorded = metrics.get_recorded();
    assert_eq!(recorded.len(), 2);

    // Verify providers were called
    assert_eq!(eth_provider.get_call_count(), 1);
    assert_eq!(bsc_provider.get_call_count(), 1);
}

#[tokio::test]
async fn test_multiple_tokens_per_account() {
    let balances = vec![
        TokenBalance {
            account_address: "0x123".to_string(),
            account_alias: "MultiToken".to_string(),
            network: "ethereum".to_string(),
            token: "ETH".to_string(),
            balance: dec!(2.0),
            threshold: dec!(1.0),
        },
        TokenBalance {
            account_address: "0x123".to_string(),
            account_alias: "MultiToken".to_string(),
            network: "ethereum".to_string(),
            token: "USDT".to_string(),
            balance: dec!(50.0),
            threshold: dec!(100.0),
        },
        TokenBalance {
            account_address: "0x123".to_string(),
            account_alias: "MultiToken".to_string(),
            network: "ethereum".to_string(),
            token: "USDC".to_string(),
            balance: dec!(200.0),
            threshold: dec!(100.0),
        },
    ];

    let provider = Arc::new(TestBalanceProvider::new("ethereum", balances));
    let notifier = Arc::new(TestNotifier::new());
    let metrics = Arc::new(TestMetricsRecorder::new());

    let account = MonitoredAccount {
        address: "0x123".to_string(),
        alias: "MultiToken".to_string(),
        network: "ethereum".to_string(),
        threshold: dec!(100.0),
        tokens: vec![
            TokenId::Native,
            TokenId::Contract("0xUSDT".to_string()),
            TokenId::Contract("0xUSDC".to_string()),
        ],
    };

    let service = BalanceMonitorService::new(
        vec![provider],
        notifier.clone(),
        metrics.clone(),
        vec![account],
        Duration::from_secs(60),
    );

    tokio::time::timeout(Duration::from_secs(1), async {
        service.check_cycle().await;
    })
    .await
    .unwrap();

    // Only USDT should trigger alert
    let alerts = notifier.get_alerts();
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].token, "USDT");

    // All three should be recorded
    let recorded = metrics.get_recorded();
    assert_eq!(recorded.len(), 3);
}

#[tokio::test]
async fn test_provider_not_found() {
    let notifier = Arc::new(TestNotifier::new());
    let metrics = Arc::new(TestMetricsRecorder::new());

    let account = MonitoredAccount {
        address: "0x123".to_string(),
        alias: "UnknownNetwork".to_string(),
        network: "unknown_network".to_string(),
        threshold: dec!(100.0),
        tokens: vec![TokenId::Native],
    };

    let service = BalanceMonitorService::new(
        vec![],
        notifier.clone(),
        metrics.clone(),
        vec![account],
        Duration::from_secs(60),
    );

    tokio::time::timeout(Duration::from_secs(1), async {
        service.check_cycle().await;
    })
    .await
    .unwrap();

    // No alerts or metrics should be recorded
    assert_eq!(notifier.get_alerts().len(), 0);
    assert_eq!(metrics.get_recorded().len(), 0);
}
