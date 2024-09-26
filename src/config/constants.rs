use crate::chains::coin::Coin;
use crate::config::env_config::get_env;

pub struct EnvConstants {
  pub pool_id: u64,
  pub token_in: Coin,
  pub token_out: Coin,
  pub gas_token: Coin,
}

pub fn get_constants() -> EnvConstants {
  let env = get_env();
  
  match env.as_str() {
    "prod" => EnvConstants {
      pool_id: 1721,
      token_in: Coin::WLibra,
      token_out: Coin::USDC,
      gas_token: Coin::OSMO
    },
    _ => EnvConstants {
      pool_id: 15,
      token_in: Coin::TOSMO,
      token_out: Coin::TUSDC,
      gas_token: Coin::TOSMO
    },
  }
}