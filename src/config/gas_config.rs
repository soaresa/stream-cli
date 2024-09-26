use crate::chains::coin::Coin;
use crate::config::env_config::get_env;

pub struct GasConfig {
  pub token: Coin,
  pub amount: u64,
  pub gas_limit: u64,
}

impl GasConfig {
  pub fn default() -> Self {
    let token = match get_env().as_str() {
      "prod" => Coin::OSMO,
      _ => Coin::TOSMO,
    };
    
    GasConfig {
      token,
      amount: 320_000,
      gas_limit: 350_000,
    }
  }
}
