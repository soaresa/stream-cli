use std::fs;
use std::env;
use std::io::Read;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use std::thread;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use serde_json::{Value, json};
use anyhow::Error;
use reqwest::Client;
use crate::configs::get_config_path;
use crate::chains::coin::Coin;
use regex::Regex;
use cosmrs::tx::Tx;
use prost::Message;


#[derive(Serialize, Deserialize, Debug)]
pub struct BroadcastedTx {
    txhash: String,
    timestamp: String,
    status_code: Option<u64>,
    pool_id: u64,
    token_in: Coin,
    token_out: Coin,
    amount: u64,
    swap_type: String,
    min_price: f64,
    tx_status: String,
    raw_log: Option<String>,
}

pub async fn broadcast_tx(
    tx: Tx, 
    sender_address: &str, 
    pool_id: u64, 
    coin_in: Coin, 
    coin_out: Coin, 
    amount: u64, 
    swap_type: &str,
    min_price: f64
) -> Result<String, anyhow::Error> {
    // Encode the transaction
    let proto_tx: cosmrs::proto::cosmos::tx::v1beta1::Tx = tx.into();
    let mut tx_bytes = Vec::new();
    proto_tx.encode(&mut tx_bytes).map_err(|e| anyhow::anyhow!("Failed to encode Tx: {}", e))?;
    let tx_base64 = base64::encode(&tx_bytes);

    // Broadcast the transaction
    let client = Client::new();
    let broadcast_url = get_osmosis_broadcast_tx_url();
    let broadcast_body = json!({
        "tx_bytes": tx_base64,
        "mode": 2
    });
    let response = client.post(broadcast_url)
        .json(&broadcast_body)
        .send()
        .await?;

    // Parse the response JSON
    let response_json: serde_json::Value = match response.json().await {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Failed to parse response JSON: {}", e);
            return Err(anyhow::anyhow!("Failed to parse response JSON: {}", e));
        }
    };
    println!(">>> Transaction broadcasted");

    // Store the broadcasted transaction
    let txhash = response_json["tx_response"]["txhash"].as_str().unwrap();
    let code = response_json["tx_response"]["code"].as_u64();
    let raw_log = response_json["tx_response"]["raw_log"].as_str().map(String::from);
    let _ = store_broadcasted_transaction(
        sender_address,
        txhash,
        code,
        raw_log,
        pool_id,
        coin_in,
        coin_out,
        amount,
        swap_type,
        min_price,
    );

    let ret = match code {
        Some(0) => {           
            // Poll the transaction status
            let res = poll_transaction_status(txhash, sender_address).await;
            match res {
                Ok(code) => {
                    match code {
                        Some(0) => {
                            "Transaction executed successfully".to_string()
                        },
                        Some(err_code) => {
                            format!("Transaction failed with code: {}", err_code)
                        },
                        None => {
                            "Transaction status unknown".to_string()
                        }
                    }
                },
                Err(_) => {
                    "Error polling transaction status".to_string()
                }
            }
        },
        Some(err_code) => {
            format!("Broadcast failed with code: {}", err_code)
        },
        None => {
            "Broadcast failed with unknown error".to_string()
        }
    };

    Ok(ret)
}

fn store_broadcasted_transaction(
    account_id: &str,
    txhash: &str,
    status_code: Option<u64>,
    raw_log: Option<String>,
    pool_id: u64,
    token_in: Coin,
    token_out: Coin,
    amount: u64,
    swap_type: &str,
    min_price: f64
) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = get_transactions_file_path()?;
    
    // Step 1: Read the file content or create a new file if it doesn't exist
    let file_content = match fs::read_to_string(&file_path) {
        Ok(content) => content,
        Err(_) => "{}".to_string(), // File not found, initialize with empty JSON object
    };
    
    // Step 2: Parse JSON content
    let mut transactions: Value = serde_json::from_str(&file_content)?;

    // Step 3: Ensure the transactions object is a JSON object
    let transactions_obj = transactions.as_object_mut().ok_or("Invalid JSON structure")?;

    // Step 4: Ensure the account_id entry exists
    let account_transactions = transactions_obj
        .entry(account_id.to_string())
        .or_insert_with(|| Value::Array(Vec::new()))
        .as_array_mut()
        .ok_or("Failed to get account transactions array")?;

    // Step 5: Create the new transaction entry
    let tx = BroadcastedTx {
        txhash: txhash.to_string(),
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs().to_string(),
        tx_status: "broadcasted".to_string(),
        status_code: status_code,
        raw_log,
        pool_id,
        token_in,
        token_out,
        amount,
        swap_type: swap_type.to_string(),
        min_price,
    };

    // Step 6: Add the new transaction to the account's transaction list
    account_transactions.push(serde_json::to_value(tx)?);

    // Step 7: Write the updated content back to the file
    fs::write(file_path, serde_json::to_string(&transactions)?)?;

    Ok(())
}

fn get_transactions(account_id: &str) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error>> {
  let file_path = get_transactions_file_path()?;
  let mut file_content = String::new();
  let mut file = fs::File::open(file_path.clone()).unwrap_or_else(|_| fs::File::create(file_path).unwrap());
  file.read_to_string(&mut file_content)?;

  let mut transactions: serde_json::Value = serde_json::from_str(&file_content)?;

  let account_transactions = transactions.as_object_mut()
      .and_then(|map| map.get_mut(account_id))
      .map(|v| v.take());

  Ok(account_transactions)
}

// Function to get the path to the wallets file
fn get_transactions_file_path() -> Result<PathBuf, Error> {
  let app_dir_path = get_config_path();
  // create dir if not exists
  if !app_dir_path.exists() {
    fs::create_dir_all(&app_dir_path)?;
  }
  Ok(app_dir_path.join("osmosis_transactions.json"))
}

pub async fn fetch_transaction_details(txhash: &str, account_id: &str) -> Result<(Option<u64>, Option<String>, Option<u64>, Option<u64>, Option<u64>), Error> {
    let client = Client::new();
    let url = get_osmosis_tx_details_url();
    let url_formated: String = url.replace("{}", txhash); 
    
    let response = client.get(&url_formated).send().await?.text().await?;
    let json: Value = serde_json::from_str(&response)?; 
    let code = json["tx_response"]["code"].as_u64();
    let raw_log = json["tx_response"]["raw_log"].as_str().map(String::from);
    let gas_used = json["tx_response"]["gas_used"].as_str()
        .and_then(|s| s.parse::<u64>().ok());

    let (tokens_in, tokens_out) = if code == Some(0) {
        let events = json["tx_response"]["events"].as_array().unwrap_or(&vec![]).to_vec();
        let mut tokens_in = None;
        let mut tokens_out = None;

        for event in events {
            match event["type"].as_str() {
                Some("token_swapped") => {
                    if event["attributes"]
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .any(|attr| attr["key"] == "sender" && attr["value"] == account_id)
                    {
                        tokens_in = event["attributes"]
                            .as_array()
                            .unwrap_or(&vec![])
                            .iter()
                            .find(|attr| attr["key"] == "tokens_in")
                            .and_then(|attr| {
                                attr["value"].as_str().and_then(|s| {
                                    // Regular expression to match leading digits
                                    let re = Regex::new(r"^\d+").unwrap();
                                    re.find(s).and_then(|m| m.as_str().parse::<u64>().ok())
                                })
                            });

                        tokens_out = event["attributes"]
                            .as_array()
                            .unwrap_or(&vec![])
                            .iter()
                            .find(|attr| attr["key"] == "tokens_out")
                            .and_then(|attr| {
                                attr["value"].as_str().and_then(|s| {
                                    // Regular expression to match leading digits
                                    let re = Regex::new(r"^\d+").unwrap();
                                    re.find(s).and_then(|m| m.as_str().parse::<u64>().ok())
                                })
                            });
                    }
                },
                _ => {}
            }
        }        

        (tokens_in, tokens_out)
    } else {
        (None, None)
    };

    Ok((code, raw_log, gas_used, tokens_in, tokens_out))
}

async fn poll_transaction_status(txhash: &str, account_id: &str) -> Result<Option<u64>, Box<dyn std::error::Error>> {
    let start_time = std::time::SystemTime::now();
    let timeout_duration = Duration::new(60, 0); // 60 seconds
    let poll_interval = Duration::new(3, 0); // 3 seconds

    loop {
        let elapsed = start_time.elapsed()?;
        if elapsed >= timeout_duration {
            update_transaction_with_timeout(txhash).await?;
            println!("!!! Transaction polling timed out for txhash: {}", txhash);
            return Ok(None);
        }

        // Fetch transaction details
        match fetch_transaction_details(txhash, account_id).await {
            Ok((code, raw_log, gas_used, tokens_in, tokens_out)) => {
                if code.is_some() {
                    // Transaction was executed
                    update_transaction_status(txhash, account_id, "executed", code, raw_log, gas_used, tokens_in, tokens_out).await?;
                    return Ok(code);
                } else {
                    println!("... Transaction not yet confirmed");
                }
            }
            Err(e) => {
                println!("!!! Error fetching transaction details: {:?}", e);
                return Ok(None)
            }
        }

        // Wait before the next polling attempt
        thread::sleep(poll_interval);
    }
}


// Function to update the transaction status in the JSON file
async fn update_transaction_status(
    txhash: &str,
    account_id: &str,
    status: &str,
    code: Option<u64>,
    raw_log: Option<String>,
    gas_used: Option<u64>,
    tokens_in: Option<u64>,
    tokens_out: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = get_transactions_file_path()?;
    
    // Read the existing transactions
    let file_content = fs::read_to_string(&file_path).unwrap_or_else(|_| "{}".to_string());
    let mut transactions: Value = serde_json::from_str(&file_content)?;

    // Find the transaction entry to update
    if let Some(account_transactions) = transactions.as_object_mut().and_then(|map| map.get_mut(account_id)) {
        if let Some(transaction) = account_transactions.as_array_mut()
            .and_then(|array| array.iter_mut().find(|tx| tx["txhash"] == txhash)) {

            // Update the transaction details
            transaction["tx_status"] = json!(status);
            transaction["status_code"] = json!(code);
            transaction["raw_log"] = json!(raw_log);
            transaction["gas_used"] = json!(gas_used);
            transaction["tokens_in"] = json!(tokens_in);
            transaction["tokens_out"] = json!(tokens_out);

            // Write the updated transactions back to the file
            fs::write(file_path, serde_json::to_string_pretty(&transactions)?)?;
        }
    }

    Ok(())
}


// Function to handle timeout scenario
async fn update_transaction_with_timeout(txhash: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Implement your logic to update the transaction with timeout error here
    update_transaction_status(txhash, "account_id", "timeout", None, None, None, None, None).await?;
    Ok(())
}

fn get_osmosis_broadcast_tx_url() -> String {
    env::var("OSMOSIS_BROADCAST_TX_URL").unwrap_or_else(|_| "https://lcd.osmosis.zone/cosmos/tx/v1beta1/txs".to_string())
}

fn get_osmosis_tx_details_url() -> String {
    env::var("OSMOSIS_TX_DETAILS_URL").unwrap_or_else(|_| "https://lcd.osmosis.zone/cosmos/tx/v1beta1/txs/{}".to_string())
}