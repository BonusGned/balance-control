use rust_decimal::Decimal;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkType {
    Evm,
    Tron,
    Solana,
}

impl NetworkType {
    pub fn from_name(name: &str) -> Self {
        match name {
            "tron" => Self::Tron,
            "solana" => Self::Solana,
            _ => Self::Evm,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenId {
    Native,
    Contract(String),
}

impl TokenId {
    pub fn parse(s: &str) -> Self {
        if s.eq_ignore_ascii_case("native") {
            Self::Native
        } else {
            Self::Contract(s.to_string())
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Self::Native => "native",
            Self::Contract(addr) => addr,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MonitoredAccount {
    pub address: String,
    pub alias: String,
    pub network: String,
    pub threshold: Decimal,
    pub tokens: Vec<TokenId>,
}

#[derive(Debug, Clone)]
pub struct TokenBalance {
    pub account_address: String,
    pub account_alias: String,
    pub network: String,
    pub token: String,
    pub balance: Decimal,
    pub threshold: Decimal,
}

impl TokenBalance {
    pub fn is_below_threshold(&self) -> bool {
        self.balance < self.threshold
    }
}

impl fmt::Display for TokenBalance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}/{}: {} (threshold: {})",
            self.network, self.account_alias, self.token, self.balance, self.threshold
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_network_type_from_name() {
        assert_eq!(NetworkType::from_name("tron"), NetworkType::Tron);
        assert_eq!(NetworkType::from_name("solana"), NetworkType::Solana);
        assert_eq!(NetworkType::from_name("ethereum"), NetworkType::Evm);
        assert_eq!(NetworkType::from_name("bsc"), NetworkType::Evm);
        assert_eq!(NetworkType::from_name("polygon"), NetworkType::Evm);
        assert_eq!(NetworkType::from_name("unknown"), NetworkType::Evm);
    }

    #[test]
    fn test_token_id_parse_native() {
        assert_eq!(TokenId::parse("native"), TokenId::Native);
        assert_eq!(TokenId::parse("Native"), TokenId::Native);
        assert_eq!(TokenId::parse("NATIVE"), TokenId::Native);
    }

    #[test]
    fn test_token_id_parse_contract() {
        let addr = "0x1234567890abcdef";
        assert_eq!(TokenId::parse(addr), TokenId::Contract(addr.to_string()));
    }

    #[test]
    fn test_token_id_display_name() {
        assert_eq!(TokenId::Native.display_name(), "native");
        let addr = "0xabcd";
        assert_eq!(TokenId::Contract(addr.to_string()).display_name(), addr);
    }

    #[test]
    fn test_token_balance_is_below_threshold() {
        let balance_below = TokenBalance {
            account_address: "0x123".to_string(),
            account_alias: "test".to_string(),
            network: "ethereum".to_string(),
            token: "USDT".to_string(),
            balance: dec!(50.0),
            threshold: dec!(100.0),
        };
        assert!(balance_below.is_below_threshold());

        let balance_above = TokenBalance {
            account_address: "0x123".to_string(),
            account_alias: "test".to_string(),
            network: "ethereum".to_string(),
            token: "USDT".to_string(),
            balance: dec!(150.0),
            threshold: dec!(100.0),
        };
        assert!(!balance_above.is_below_threshold());

        let balance_equal = TokenBalance {
            account_address: "0x123".to_string(),
            account_alias: "test".to_string(),
            network: "ethereum".to_string(),
            token: "USDT".to_string(),
            balance: dec!(100.0),
            threshold: dec!(100.0),
        };
        assert!(!balance_equal.is_below_threshold());
    }

    #[test]
    fn test_token_balance_display() {
        let balance = TokenBalance {
            account_address: "0x123".to_string(),
            account_alias: "MyWallet".to_string(),
            network: "ethereum".to_string(),
            token: "USDT".to_string(),
            balance: dec!(50.5),
            threshold: dec!(100.0),
        };
        let display = format!("{}", balance);
        assert!(display.contains("[ethereum]"));
        assert!(display.contains("MyWallet/USDT"));
        assert!(display.contains("50.5"));
        assert!(display.contains("threshold"));
        assert!(display.contains("100"));
    }
}
