use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::domain::model::{MonitoredAccount, TokenBalance, TokenId};
use crate::domain::ports::BalanceProvider;
use crate::tokens;

const TRX_DECIMALS: u32 = 6;

pub struct TronHttpProvider {
    client: reqwest::Client,
    base_url: String,
    network_name: String,
}

impl TronHttpProvider {
    pub fn new(network_name: String, base_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            network_name,
        }
    }

    async fn fetch_trx_balance(&self, address: &str) -> anyhow::Result<Decimal> {
        let url = format!("{}/v1/accounts/{}", self.base_url, address);
        let resp: serde_json::Value = self.client.get(&url).send().await?.json().await?;
        let balance = resp["data"][0]["balance"].as_u64().unwrap_or(0);
        Ok(Decimal::from(balance) / Decimal::from(10u64.pow(TRX_DECIMALS)))
    }

    async fn fetch_trc20_balance(
        &self,
        owner: &str,
        contract: &str,
    ) -> anyhow::Result<Decimal> {
        let owner_hex = tron_base58_to_eth_hex(owner)?;
        let parameter = format!("{owner_hex:0>64}");

        let body = serde_json::json!({
            "owner_address": owner,
            "contract_address": contract,
            "function_selector": "balanceOf(address)",
            "parameter": parameter,
            "visible": true
        });

        let url = format!("{}/wallet/triggerconstantcontract", self.base_url);
        let resp: serde_json::Value = self.client.post(&url).json(&body).send().await?.json().await?;

        let hex_result = resp["constant_result"][0]
            .as_str()
            .unwrap_or("0");
        let raw = u128::from_str_radix(hex_result.trim_start_matches("0x"), 16).unwrap_or(0);

        let decimals = tokens::find_decimals(&self.network_name, contract).unwrap_or(6);
        Ok(Decimal::from(raw) / Decimal::from(10u64.pow(decimals as u32)))
    }
}

#[async_trait]
impl BalanceProvider for TronHttpProvider {
    async fn fetch_balances(
        &self,
        account: &MonitoredAccount,
    ) -> anyhow::Result<Vec<TokenBalance>> {
        let mut balances = Vec::with_capacity(account.tokens.len());

        for token in &account.tokens {
            let (balance, display) = match token {
                TokenId::Native => (
                    self.fetch_trx_balance(&account.address).await?,
                    "TRX".to_string(),
                ),
                TokenId::Contract(addr) => {
                    let bal = self
                        .fetch_trc20_balance(&account.address, addr)
                        .await?;
                    let name = tokens::find_symbol(&self.network_name, addr)
                        .map(String::from)
                        .unwrap_or_else(|| addr.clone());
                    (bal, name)
                }
            };

            balances.push(TokenBalance {
                account_address: account.address.clone(),
                account_alias: account.alias.clone(),
                network: self.network_name.clone(),
                token: display,
                balance,
                threshold: account.threshold,
            });
        }

        Ok(balances)
    }

    fn supports_network(&self, network: &str) -> bool {
        self.network_name == network
    }
}

fn tron_base58_to_eth_hex(address: &str) -> anyhow::Result<String> {
    let decoded = bs58::decode(address).into_vec()?;
    if decoded.len() < 5 {
        anyhow::bail!("Invalid Tron address: too short");
    }
    // Skip first byte (0x41 prefix) and last 4 bytes (checksum)
    let eth_bytes = &decoded[1..decoded.len() - 4];
    Ok(hex::encode(eth_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tron_base58_to_eth_hex() {
        // Valid Tron address
        let result = tron_base58_to_eth_hex("TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t");
        assert!(result.is_ok());
        let hex = result.unwrap();
        assert!(!hex.is_empty());
        assert_eq!(hex.len(), 40); // 20 bytes = 40 hex chars
    }

    #[test]
    fn test_tron_base58_to_eth_hex_invalid() {
        let result = tron_base58_to_eth_hex("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_supports_network() {
        let provider = TronHttpProvider::new("tron".to_string(), "https://api.trongrid.io");
        assert!(provider.supports_network("tron"));
        assert!(!provider.supports_network("ethereum"));
    }
}
