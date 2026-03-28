use async_trait::async_trait;
use rust_decimal::Decimal;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use crate::domain::model::{MonitoredAccount, TokenBalance, TokenId};
use crate::domain::ports::BalanceProvider;
use crate::tokens;

const SOL_DECIMALS: u32 = 9;
const SPL_TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

pub struct SolanaBalanceProvider {
    client: RpcClient,
    network_name: String,
}

impl SolanaBalanceProvider {
    pub fn new(network_name: String, rpc_url: &str) -> Self {
        Self {
            client: RpcClient::new(rpc_url.to_string()),
            network_name,
        }
    }

    async fn fetch_native_balance(&self, address: &str) -> anyhow::Result<Decimal> {
        let pubkey = Pubkey::from_str(address)?;
        let lamports = self.client.get_balance(&pubkey).await?;
        Ok(Decimal::from(lamports) / Decimal::from(10u64.pow(SOL_DECIMALS)))
    }

    async fn fetch_spl_balance(&self, owner: &str, mint: &str) -> anyhow::Result<Decimal> {
        let owner_pubkey = Pubkey::from_str(owner)?;
        let mint_pubkey = Pubkey::from_str(mint)?;
        let token_program = Pubkey::from_str(SPL_TOKEN_PROGRAM)?;

        let ata = spl_associated_token_account::get_associated_token_address_with_program_id(
            &owner_pubkey,
            &mint_pubkey,
            &token_program,
        );

        match self.client.get_token_account_balance(&ata).await {
            Ok(balance) => {
                let decimals = tokens::find_decimals(&self.network_name, mint).unwrap_or(6);
                Decimal::from_str(&balance.ui_amount_string)
                    .or_else(|_| {
                        let raw = u64::from_str(&balance.amount).unwrap_or(0);
                        Ok(Decimal::from(raw) / Decimal::from(10u64.pow(decimals as u32)))
                    })
                    .map_err(|e: rust_decimal::Error| anyhow::anyhow!(e))
            }
            Err(_) => Ok(Decimal::ZERO),
        }
    }
}

#[async_trait]
impl BalanceProvider for SolanaBalanceProvider {
    async fn fetch_balances(
        &self,
        account: &MonitoredAccount,
    ) -> anyhow::Result<Vec<TokenBalance>> {
        let mut balances = Vec::with_capacity(account.tokens.len());

        for token in &account.tokens {
            let (balance, display) = match token {
                TokenId::Native => (
                    self.fetch_native_balance(&account.address).await?,
                    "SOL".to_string(),
                ),
                TokenId::Contract(mint) => {
                    let bal = self.fetch_spl_balance(&account.address, mint).await?;
                    let name = tokens::find_symbol(&self.network_name, mint)
                        .map(String::from)
                        .unwrap_or_else(|| mint.clone());
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
