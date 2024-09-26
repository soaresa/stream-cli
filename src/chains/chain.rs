use serde::{Deserialize, Serialize};
use std::fmt;
use crate::config::CONFIG;

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
  pub fn chain_id(&self) -> &String {
    match self {
      ChainType::Osmosis => &CONFIG.osmosis_chain_id
      // Add other chains as needed
    }
  }
}