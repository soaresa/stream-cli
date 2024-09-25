use serde::{Deserialize, Serialize};
use std::fmt;
use crate::utils::format_token_amount_with_denom;

// Enum for coins with associated denoms
#[derive(Serialize, Deserialize, Clone, Debug, Copy, PartialEq, Eq )]
pub enum Coin {
    WLibra,
    USDC,
    OSMO,
    TOSMO, // test
    TUSDC, // test - pool id TUSDC / TOSMO - 67
}

// Implement Display trait for Coin
impl fmt::Display for Coin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let coin_name = match self {
            Coin::WLibra => "WLibra",
            Coin::USDC => "USDC",
            Coin::OSMO => "OSMO",
            Coin::TOSMO => "TOSMO",
            Coin::TUSDC => "TUSDC",
        };
        write!(f, "{}", coin_name)
    }
}

impl Coin {
    // Method to get the denomination as a string
    pub fn denom(&self) -> &'static str {
        match self {
            Coin::WLibra => "factory/osmo19hdqma2mj0vnmgcxag6ytswjnr8a3y07q7e70p/wLIBRA",
            Coin::USDC => "ibc/498A0751C798A0D9A389AA3691123DADA57DAA4FE165D5C75894505B876BA6E4",
            Coin::OSMO => "uosmo",
            Coin::TOSMO => "uosmo",
            Coin::TUSDC => "factory/osmo109ns4u04l44kqdkvp876hukd3hxz8zzm7809el/uusdc",
            // Add more cases as needed
        }
    }
}

// Struct for holding coin balances
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CoinAmount {
    pub coin: Coin,
    pub amount: u64,
}

impl fmt::Display for CoinAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ret = format_token_amount_with_denom(self.amount, &self.coin.to_string());
        write!(f, "{}", ret)
    }
}
