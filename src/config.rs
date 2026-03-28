use std::collections::HashMap;
use std::path::Path;

use rust_decimal::Decimal;
use serde::Deserialize;

use crate::domain::model::{MonitoredAccount, TokenId};

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub settings: Settings,
    pub networks: HashMap<String, NetworkConfig>,
    pub accounts: Vec<AccountConfig>,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub check_interval_secs: u64,
    pub prometheus_port: u16,
    pub telegram: TelegramConfig,
}

#[derive(Debug, Deserialize)]
pub struct TelegramConfig {
    pub token: String,
    pub chat_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct NetworkConfig {
    pub rpc_url: String,
    #[allow(dead_code)]
    pub mode: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AccountConfig {
    pub address: String,
    pub alias: String,
    pub network: String,
    pub threshold: Decimal,
    pub tokens: Vec<String>,
}

impl From<&AccountConfig> for MonitoredAccount {
    fn from(cfg: &AccountConfig) -> Self {
        Self {
            address: cfg.address.clone(),
            alias: cfg.alias.clone(),
            network: cfg.network.clone(),
            threshold: cfg.threshold,
            tokens: cfg.tokens.iter().map(|t| TokenId::parse(t)).collect(),
        }
    }
}

pub fn load(path: impl AsRef<Path>) -> anyhow::Result<AppConfig> {
    let contents = std::fs::read_to_string(path)?;
    let config: AppConfig = serde_yaml::from_str(&contents)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_account_config_to_monitored_account() {
        let cfg = AccountConfig {
            address: "0x123".to_string(),
            alias: "TestWallet".to_string(),
            network: "ethereum".to_string(),
            threshold: dec!(100.0),
            tokens: vec!["native".to_string(), "0xabc".to_string()],
        };

        let account = MonitoredAccount::from(&cfg);
        assert_eq!(account.address, "0x123");
        assert_eq!(account.alias, "TestWallet");
        assert_eq!(account.network, "ethereum");
        assert_eq!(account.threshold, dec!(100.0));
        assert_eq!(account.tokens.len(), 2);
        assert_eq!(account.tokens[0], TokenId::Native);
        assert_eq!(account.tokens[1], TokenId::Contract("0xabc".to_string()));
    }

    #[test]
    fn test_load_config() {
        let yaml = r#"
settings:
  check_interval_secs: 60
  prometheus_port: 9090
  telegram:
    token: "test_token"
    chat_id: 123456

networks:
  ethereum:
    rpc_url: "https://eth.example.com"

accounts:
  - address: "0x123"
    alias: "Wallet1"
    network: "ethereum"
    threshold: 100.0
    tokens: ["native"]
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(yaml.as_bytes()).unwrap();
        file.flush().unwrap();

        let config = load(file.path()).unwrap();
        assert_eq!(config.settings.check_interval_secs, 60);
        assert_eq!(config.settings.prometheus_port, 9090);
        assert_eq!(config.settings.telegram.token, "test_token");
        assert_eq!(config.settings.telegram.chat_id, 123456);
        assert_eq!(config.networks.len(), 1);
        assert_eq!(config.accounts.len(), 1);
        assert_eq!(config.accounts[0].alias, "Wallet1");
    }
}
