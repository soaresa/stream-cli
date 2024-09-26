use log::{error, info, warn};
use crate::config::gas_config::GasConfig;
use crate::chains::coin::Coin;
use crate::chains::osmosis::osmosis_pool_service;
use crate::chains::osmosis::osmosis_key_service::Signer;
use crate::chains::osmosis::osmosis_pool_service::fetch_coin_price;
use crate::chains::osmosis::osmosis_account_service::fetch_balances;
use crate::chains::coin::CoinAmount;
use anyhow::{anyhow, Result};

/// the trade tasks that the stream processes
pub struct TradeTask {
    pool_id: u64,
    token_in: Coin,
    token_out: Coin,
    amount: u64,
    swap_type: &'static str,
    min_price: f64,
}

impl TradeTask {
    pub fn new(
        pool_id: u64,
        token_in: Coin,
        token_out: Coin,
        amount: u64,
        swap_type: &'static str,
        min_price: f64,
    ) -> Self {
        TradeTask {
            pool_id,
            token_in,
            token_out,
            amount,
            swap_type,
            min_price,
        }
    }
}

impl TradeTask {
    pub async fn execute(&self, signer: &Signer) -> Result<bool, anyhow::Error> {
        // Note: some checks can be removed to run faster
        
        // 1. Check coin price
        let price = match fetch_coin_price(self.pool_id).await {
            Ok(value) => {
                value
            }
            Err(e) => {
                error!("!!! 1. Error fetching coin price: {:?}", e);
                return Ok(false);
            }
        };
        if price < self.min_price {
            warn!("!!! 1. Current price {} is less than min price {} to perform swap", price, self.min_price);
            return Ok(false);
        }
        info!(">>> 1. Current price {} is above min price {}", price, self.min_price);

        // Fetch account balances
        let balances = match fetch_balances(signer.get_account_address(), None).await {
            Ok(balances) => balances,
            Err(e) => {
                error!("!!! 2. Error fetching account balances: {:?}", e);
                return Ok(false);
            }
        };

        // 2. Check account balance for the token to swap
        let trade_amount = match self.swap_type {
            "amount_out" => (self.amount as f64 / price) as u64,
            "amount_in" => (self.amount as f64 * price) as u64,
            _ => {
                error!("!!! 2. Invalid swap type: {}", self.swap_type);
                return Ok(false);
            }
        };
        
        if let Err(e) = has_sufficient_balance(&balances, self.token_in.denom(), trade_amount) {
            error!("{}", e);
            return Ok(false);
        }
        info!(">>> 2. Account has enough balance to perform swap");

        // 3. Ensure account has enough balance to pay for fees
        // TODO: Implement gas station
        let gas_config = GasConfig::default();
        if let Err(e) = has_sufficient_balance(&balances, gas_config.token.denom(), gas_config.gas_limit) {
            error!("{}", e);
            return Ok(false);
        }
        info!(">>> 3. Account has enough gas balance to cover fees");
     
        // 4. Perform the swap
        osmosis_pool_service::perform_swap(
            signer,
            self.pool_id,
            self.token_in,
            self.token_out,
            self.amount,
            self.swap_type,
            self.min_price,
        ).await
    }
}

// Helper function
fn has_sufficient_balance(balances: &[CoinAmount], denom: &str, required_amount: u64) -> Result<(), anyhow::Error> {
    if let Some(balance) = balances.iter().find(|b| b.coin.denom() == denom) {
        if balance.amount < required_amount {
            return Err(anyhow!("Insufficient balance for token: {}", denom));
        }
    } else {
        return Err(anyhow!("No balance found for token: {}", denom));
    }
    Ok(())
}
