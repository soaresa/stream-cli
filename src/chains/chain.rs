use serde::{Deserialize, Serialize};
use std::fmt;
use std::env;

// Enum for different chain types
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum ChainType {
  Osmosis,
  // Add other chains as needed
}

impl fmt::Display for ChainType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ChainType::Osmosis => write!(f, "Osmosis"),
      // Add other chains as needed
    }
  }
}

// implement an function to return the chain id 
impl ChainType {
  pub fn chain_id(&self) -> String {
    match self {
      ChainType::Osmosis => env::var("OSMOSIS_CHAIN_ID").unwrap_or_else(|_| "osmosis-1".to_string()),
      // Add other chains as needed
    }
  }
}