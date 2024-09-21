use dotenv::{dotenv, from_filename};
use std::env;
use dirs_next::config_dir;
use std::path::PathBuf;

pub fn initialize() {
  // Load environment variables from the .env file
  dotenv().ok();

  // Read the ENVIRONMENT variable
  let environment = get_env();
  println!(">>> Environment: {}", &environment);

  // Load environment variables from the appropriate .env file
  let env_file = match environment.as_str() {
    "test" => "src/.env.test",
    _ => "src/.env.prod",
  };
  from_filename(env_file).ok(); // Load the specified .env file
}

pub fn get_env() -> String {
  env::var("ENVIRONMENT").unwrap_or_else(|_| "prod".to_string())
}

pub fn get_config_path() -> PathBuf {
  let config_path = match get_env().as_str() {
      "test" => default_config_path().join("stream/test"),
      _ => default_config_path().join("stream/prod"),
  };

  // Create the directory if it doesn't exist
  if !config_path.exists() {
      std::fs::create_dir_all(&config_path).unwrap();
  }

  config_path
}

pub fn default_config_path() -> PathBuf {
  // Use the system-specific config directory
  config_dir().unwrap_or_else(|| {
      // Fallback in case the system config dir is unavailable
      PathBuf::from(".")
  })
}