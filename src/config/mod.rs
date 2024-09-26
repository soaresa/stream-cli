use serde::Deserialize;
use once_cell::sync::Lazy;
use std::sync::Arc;
use std::path::PathBuf;
use dirs_next::config_dir;
use std::fs;

use crate::chains::coin::Coin;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub environment: String,
    pub gas_config: GasConfig,
    pub env_constants: EnvConstants,
    pub osmosis_chain_id: String,
    pub osmosis_status_url: String,
    pub osmosis_account_info_url: String,
    pub osmosis_broadcast_tx_url: String,
    pub osmosis_pool_price_url: String,
    pub osmosis_account_balances_url: String,
    pub osmosis_tx_details_url: String,
}

#[derive(Debug, Deserialize)]
pub struct GasConfig {
    pub token: Coin,
    pub amount: u64,
    pub gas_limit: u64,
}

#[derive(Debug, Deserialize)]
pub struct EnvConstants {
    pub pool_id: u64,
    pub token_in: Coin,
    pub token_out: Coin,
    pub gas_token: Coin,
}

impl Config {
    fn from_env() -> Result<Self, config::ConfigError> {
        dotenv::dotenv().ok();

        let environment = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "prod".into());
        println!("Loading configuration for environment: {}", &environment);

        let mut builder = config::Config::builder()
            // Load environment-specific settings
            .add_source(config::File::with_name(&format!("src/config/{}", environment)).required(true))
            // Allow environment variables to override (prefix with 'APP_')
            .add_source(config::Environment::with_prefix("APP").separator("__"));

        // Insert the environment into the configuration
        builder = builder.set_override("environment", environment.clone())?;

        // Build the configuration
        let config = builder.build()?;

        // Deserialize into the Config struct
        config.try_deserialize().map_err(|e| {
            println!("Deserialization error: {}", e);
            e
        })
    }
}


// Global static instance of Config
pub static CONFIG: Lazy<Arc<Config>> = Lazy::new(|| {
    let config = Config::from_env().expect("Failed to load configuration");
    Arc::new(config)
});

/// Returns the default configuration path based on the operating system.
fn default_config_path() -> PathBuf {
  // Use the system-specific config directory
  config_dir().unwrap_or_else(|| {
      // Fallback in case the system config dir is unavailable
      PathBuf::from(".")
  })
}

/// Gets the configuration path based on the environment.
pub fn get_config_path() -> PathBuf {
  let config_path = match CONFIG.environment.as_str() {
      "test" => default_config_path().join("stream/test"),
      _ => default_config_path().join("stream/prod"),
  };

  // Create the directory if it doesn't exist
  if !config_path.exists() {
      fs::create_dir_all(&config_path).unwrap();
  }

  config_path
}