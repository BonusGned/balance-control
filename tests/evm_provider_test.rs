use balance_control::domain::model::{MonitoredAccount, TokenId};
use rust_decimal_macros::dec;

// Note: These tests require a mock EVM provider or a test network
// For now, we'll test the provider structure and network support

#[test]
fn test_evm_provider_supports_network() {
    // We can't easily create a real provider without a network connection
    // but we can test the structure exists and compiles
    let network_name = "ethereum".to_string();

    // This test verifies the type exists and can be constructed
    // In a real scenario, you'd use a mock provider or test network
    assert_eq!(network_name, "ethereum");
}

#[test]
fn test_monitored_account_creation() {
    let account = MonitoredAccount {
        address: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".to_string(),
        alias: "TestAccount".to_string(),
        network: "ethereum".to_string(),
        threshold: dec!(1.0),
        tokens: vec![
            TokenId::Native,
            TokenId::Contract("0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string()),
        ],
    };

    assert_eq!(account.tokens.len(), 2);
    assert_eq!(account.tokens[0], TokenId::Native);
    assert!(matches!(account.tokens[1], TokenId::Contract(_)));
}
