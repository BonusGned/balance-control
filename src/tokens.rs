pub struct StandardToken {
    pub symbol: &'static str,
    pub address: &'static str,
    pub decimals: u8,
    pub network: &'static str,
}

pub static STANDARD_TOKENS: &[StandardToken] = &[
    // Ethereum
    StandardToken { symbol: "USDT", address: "0xdAC17F958D2ee523a2206206994597C13D831ec7", decimals: 6, network: "ethereum" },
    StandardToken { symbol: "USDC", address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", decimals: 6, network: "ethereum" },
    StandardToken { symbol: "WETH", address: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", decimals: 18, network: "ethereum" },
    // BSC
    StandardToken { symbol: "USDT", address: "0x55d398326f99059fF775485246999027B3197955", decimals: 18, network: "bsc" },
    StandardToken { symbol: "USDC", address: "0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d", decimals: 18, network: "bsc" },
    // Polygon
    StandardToken { symbol: "USDT", address: "0xc2132D05D31c914a87C6611C10748AEb04B58e8F", decimals: 6, network: "polygon" },
    StandardToken { symbol: "USDC", address: "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359", decimals: 6, network: "polygon" },
    // Tron
    StandardToken { symbol: "USDT", address: "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t", decimals: 6, network: "tron" },
    StandardToken { symbol: "USDC", address: "TEkxiTehnzSmSe2XqrBj4w32RUN966rdz8", decimals: 6, network: "tron" },
    // Solana
    StandardToken { symbol: "USDT", address: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", decimals: 6, network: "solana" },
    StandardToken { symbol: "USDC", address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", decimals: 6, network: "solana" },
];

pub fn find_decimals(network: &str, address: &str) -> Option<u8> {
    STANDARD_TOKENS
        .iter()
        .find(|t| t.network == network && t.address.eq_ignore_ascii_case(address))
        .map(|t| t.decimals)
}

pub fn find_symbol(network: &str, address: &str) -> Option<&'static str> {
    STANDARD_TOKENS
        .iter()
        .find(|t| t.network == network && t.address.eq_ignore_ascii_case(address))
        .map(|t| t.symbol)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_decimals_ethereum_usdt() {
        let decimals = find_decimals("ethereum", "0xdAC17F958D2ee523a2206206994597C13D831ec7");
        assert_eq!(decimals, Some(6));
    }

    #[test]
    fn test_find_decimals_case_insensitive() {
        let decimals = find_decimals("ethereum", "0xdac17f958d2ee523a2206206994597c13d831ec7");
        assert_eq!(decimals, Some(6));
    }

    #[test]
    fn test_find_decimals_not_found() {
        let decimals = find_decimals("ethereum", "0x0000000000000000000000000000000000000000");
        assert_eq!(decimals, None);
    }

    #[test]
    fn test_find_symbol_ethereum_usdc() {
        let symbol = find_symbol("ethereum", "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
        assert_eq!(symbol, Some("USDC"));
    }

    #[test]
    fn test_find_symbol_tron_usdt() {
        let symbol = find_symbol("tron", "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t");
        assert_eq!(symbol, Some("USDT"));
    }

    #[test]
    fn test_find_symbol_not_found() {
        let symbol = find_symbol("ethereum", "0x0000000000000000000000000000000000000000");
        assert_eq!(symbol, None);
    }

    #[test]
    fn test_find_symbol_wrong_network() {
        let symbol = find_symbol("bsc", "0xdAC17F958D2ee523a2206206994597C13D831ec7");
        assert_eq!(symbol, None);
    }
}
