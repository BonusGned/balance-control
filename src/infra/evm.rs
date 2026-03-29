use std::str::FromStr;

use alloy::{
    primitives::{address, Address, U256},
    providers::Provider,
    sol,
    sol_types::SolCall,
};
use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::domain::model::{MonitoredAccount, TokenBalance, TokenId};
use crate::domain::ports::BalanceProvider;
use crate::tokens;

const MULTICALL3: Address = address!("cA11bde05977b3631167028862bE2a173976CA11");
const DEFAULT_ERC20_DECIMALS: u8 = 18;
const NATIVE_DECIMALS: u8 = 18;

sol! {
    #[sol(rpc)]
    contract IERC20 {
        function balanceOf(address owner) external view returns (uint256);
    }
}

sol! {
    #[sol(rpc)]
    contract IMulticall3 {
        struct Call3 {
            address target;
            bool allowFailure;
            bytes callData;
        }

        struct Result {
            bool success;
            bytes returnData;
        }

        function aggregate3(Call3[] calldata calls) external payable returns (Result[] memory returnData);
    }
}

pub struct EvmBalanceProvider<P> {
    provider: P,
    network_name: String,
}

impl<P: Provider + Send + Sync> EvmBalanceProvider<P> {
    pub fn new(network_name: String, provider: P) -> Self {
        Self {
            provider,
            network_name,
        }
    }

    async fn fetch_native_balance(&self, owner: Address) -> anyhow::Result<Decimal> {
        let wei = self.provider.get_balance(owner).await?;
        Ok(u256_to_decimal(wei, NATIVE_DECIMALS))
    }

    async fn fetch_erc20_balances_batched(
        &self,
        owner: Address,
        token_addresses: &[Address],
    ) -> anyhow::Result<Vec<U256>> {
        if token_addresses.is_empty() {
            return Ok(vec![]);
        }

        let calls: Vec<IMulticall3::Call3> = token_addresses
            .iter()
            .map(|&target| IMulticall3::Call3 {
                target,
                allowFailure: true,
                callData: IERC20::balanceOfCall { owner }.abi_encode().into(),
            })
            .collect();

        let multicall = IMulticall3::new(MULTICALL3, &self.provider);
        let results = multicall.aggregate3(calls).call().await?;

        results
            .iter()
            .map(|r| {
                if r.success && r.returnData.len() >= 32 {
                    Ok(IERC20::balanceOfCall::abi_decode_returns(&r.returnData)
                        .unwrap_or(U256::ZERO))
                } else {
                    Ok(U256::ZERO)
                }
            })
            .collect()
    }
}

#[async_trait]
impl<P: Provider + Send + Sync + 'static> BalanceProvider for EvmBalanceProvider<P> {
    async fn fetch_balances(
        &self,
        account: &MonitoredAccount,
    ) -> anyhow::Result<Vec<TokenBalance>> {
        let owner = Address::from_str(&account.address)?;
        let mut balances = Vec::with_capacity(account.tokens.len());

        let mut erc20_addrs = Vec::new();
        let mut erc20_indices = Vec::new();

        for (i, token) in account.tokens.iter().enumerate() {
            match token {
                TokenId::Native => {
                    let balance = self.fetch_native_balance(owner).await?;
                    balances.push(TokenBalance {
                        account_address: account.address.clone(),
                        account_alias: account.alias.clone(),
                        network: self.network_name.clone(),
                        token: "native".to_string(),
                        balance,
                        threshold: account.threshold,
                    });
                }
                TokenId::Contract(addr) => {
                    erc20_addrs.push(Address::from_str(addr)?);
                    erc20_indices.push(i);
                }
            }
        }

        let raw_balances = self
            .fetch_erc20_balances_batched(owner, &erc20_addrs)
            .await?;

        for (j, raw) in raw_balances.into_iter().enumerate() {
            let idx = erc20_indices[j];
            let addr_str = format!("{}", erc20_addrs[j]);
            let decimals = tokens::find_decimals(&self.network_name, &addr_str)
                .unwrap_or(DEFAULT_ERC20_DECIMALS);
            let display = tokens::find_symbol(&self.network_name, &addr_str)
                .map(String::from)
                .unwrap_or_else(|| account.tokens[idx].display_name().to_string());

            balances.push(TokenBalance {
                account_address: account.address.clone(),
                account_alias: account.alias.clone(),
                network: self.network_name.clone(),
                token: display,
                balance: u256_to_decimal(raw, decimals),
                threshold: account.threshold,
            });
        }

        Ok(balances)
    }

    fn supports_network(&self, network: &str) -> bool {
        self.network_name == network
    }
}

fn u256_to_decimal(value: U256, decimals: u8) -> Decimal {
    let raw = Decimal::from_str(&value.to_string()).unwrap_or(Decimal::ZERO);
    let divisor = Decimal::from(10u64.pow(decimals as u32));
    raw / divisor
}
