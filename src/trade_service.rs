use log::{error, info, warn};
use crate::chains::coin::Coin;
use crate::chains::osmosis::osmosis_pool_service;
use crate::chains::osmosis::osmosis_key_service::Signer;
use crate::chains::osmosis::osmosis_pool_service::fetch_coin_price;
use crate::chains::osmosis::osmosis_account_service::fetch_account_balance;

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

        // 2. Check account balance
        match fetch_account_balance(signer.get_account_address() , self.token_in).await {
            Ok(balance) => {
                let trade_amount = match self.swap_type {
                    "amount_out" => (self.amount as f64 / price) as u64,
                    "amount_in" => (self.amount as f64 * price) as u64,
                    _ => {
                        error!("!!! 2. Invalid swap type: {}", self.swap_type);
                        return Ok(false);
                    }
                };

                if balance < trade_amount {
                    warn!("!!! 2. Insufficient balance to perform swap");
                    return Ok(false);
                }
                info!(">>> 2. Account has enough balance to perform swap");
            }
            Err(e) => {
                error!("!!! 2. Error fetching account balance: {:?}", e);
                return Ok(false);
            }
        }
        
        // 3. Ensure account has enough balance to pay for fees
        // TODO swamp coins to pay for fees
        
        // 4. Performa swap
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
