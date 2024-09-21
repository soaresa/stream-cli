use serde::Deserialize;
use std::error::Error as StdError;
use reqwest;
use crate::chains::coin::Coin;
use crate::chains::chain::ChainType;
use crate::chains::osmosis::osmosis_key_service::Signer;
use crate::chains::osmosis::osmosis_account_service::fetch_account_info;
use super::osmosis_transaction::{store_broadcasted_transaction, poll_transaction_status};

use osmosis_std::types::osmosis::gamm::v1beta1::{MsgSwapExactAmountOut, SwapAmountOutRoute};
use osmosis_std::types::cosmos::base::v1beta1::Coin as OsmosisCoin;

use cosmrs::tendermint::{block::Height, chain::Id};
use cosmrs::tx::{Body, Fee, AuthInfo, SignDoc, Tx};
use cosmrs::Any;
use cosmrs::Coin as CosmosCoin;
use cosmrs::Decimal;

use anyhow::Result;
use prost::Message;

use reqwest::Client;
use serde_json::json;
use base64;
use std::env;


fn msg_to_any(msg: MsgSwapExactAmountOut) -> Result<Any, Box<dyn std::error::Error>> {
    Ok(Any {
        type_url: "/osmosis.gamm.v1beta1.MsgSwapExactAmountOut".to_string(),
        value: msg.encode_to_vec(), // This should work if MsgSwapExactAmountOut implements `prost::Message`
    })
}

pub async fn perform_swap(
    signer: &Signer,
    pool_id: u64,
    coin_in: Coin,
    coin_out: Coin,    
    amount_out: u64,
    min_price: f64,
) -> Result<String, anyhow::Error> {

    let sender_address = signer.get_account_address();
    
    // Step 1. Calc max token in amount
    let token_in_max_amount: u64 = (amount_out as f64 / min_price) as u64;
    println!(">>> Token in max amount: {}", token_in_max_amount);  

    // Step 2. Create swap message
    let msg_swap = MsgSwapExactAmountOut {
        sender: sender_address.to_string(),
        routes: vec![SwapAmountOutRoute {
            pool_id,
            token_in_denom: coin_in.denom().parse().map_err(|e| anyhow::anyhow!("Failed to parse coin denom: {}", e))?,
        }],
        token_out: Some(OsmosisCoin {
            denom: coin_out.denom().parse().map_err(|e| anyhow::anyhow!("Failed to parse coin denom: {}", e))?,
            amount: amount_out.to_string(),
        }),
        token_in_max_amount: token_in_max_amount.to_string(),
    };
    let any_msg = msg_to_any(msg_swap).map_err(|e| anyhow::anyhow!("Failed to convert message to Any: {}", e))?;

    // Step 3. Get the current block height
    let current_height = get_current_block_height().await.map_err(|e| anyhow::anyhow!("Failed to get current block height: {}", e))?;
    let timeout_height = current_height + 1000;  // Set a future timeout height

    // Step 4. Create TxBody
    let tx_body = Body {
        messages: vec![any_msg],
        memo: "Trade Stream".to_string(),
        timeout_height: Height::try_from(timeout_height).unwrap(),
        extension_options: vec![],
        non_critical_extension_options: vec![],
    };

    // Step 5. Fetch account sequence and get signer info
    let (account_number, sequence) = fetch_account_info(sender_address).await.map_err(|e| anyhow::anyhow!("Failed to fetch account info: {}", e))?;
    let signer_info = signer.create_signer_info(sequence);
    
    // Step 6: Create AuthInfo with fee details
    let fee = Fee::from_amount_and_gas(CosmosCoin {
        denom: "uosmo".parse().unwrap(),
        amount: Decimal::from(320_000u64),
    }, 350_000);
    let auth_info = AuthInfo {
        signer_infos: vec![signer_info],
        fee,
    };  

    // Step 7: Create and sign the doc
    let comos_id = ChainType::Osmosis.chain_id();
    let chain_id = Id::try_from(comos_id)?;
    let sign_doc = SignDoc::new(&tx_body, &auth_info, &chain_id, account_number).map_err(|e| anyhow::anyhow!("Failed to create SignDoc: {}", e))?;
    let tx_bytes = signer.sign_doc(sign_doc).map_err(|e| anyhow::anyhow!("Failed to sign the transaction: {}", e))?;

    // Step 8: Create and broadcast the transaction
    let tx_parsed = Tx::from_bytes(&tx_bytes).map_err(|e| anyhow::anyhow!("Failed to parse transaction bytes: {}", e))?;
    let response = broadcast_tx(tx_parsed).await?;
    println!(">>> Transaction broadcasted: {}", response);

    // Step 9: Store the transaction details
    let response_json: serde_json::Value = serde_json::from_str(&response)?;
    let txhash = response_json["tx_response"]["txhash"].as_str().unwrap();
    let _ = store_broadcasted_transaction(
        sender_address,
        txhash,
        pool_id,
        coin_in,
        coin_out,
        amount_out,
        min_price,
    );

    // Step 10: Poll the transaction status
    let _ = poll_transaction_status(txhash, sender_address).await;

    Ok(response)
}

#[derive(Deserialize)]
struct SyncInfo {
    latest_block_height: String,
}

#[derive(Deserialize)]
struct StatusResponse {
    result: SyncInfoWrapper,
}

#[derive(Deserialize)]
struct SyncInfoWrapper {
    sync_info: SyncInfo,
}

pub async fn get_current_block_height() -> Result<u64, Box<dyn StdError>> {
    let client = Client::new();
    let url = get_osmosis_rpc_url();
    let res = client
        .get(url)  // Replace with the actual RPC endpoint URL
        .send()
        .await?
        .json::<StatusResponse>()
        .await?;

    let height = res.result.sync_info.latest_block_height.parse::<u64>()?;
    Ok(height)
}


async fn broadcast_tx(tx: Tx) -> Result<String, anyhow::Error> {
    let proto_tx: cosmrs::proto::cosmos::tx::v1beta1::Tx = tx.into();
    let mut tx_bytes = Vec::new();
    proto_tx.encode(&mut tx_bytes).map_err(|e| anyhow::anyhow!("Failed to encode Tx: {}", e))?;

    let tx_base64 = base64::encode(&tx_bytes);

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

    let response_text = response.text().await?;

    Ok(response_text)
}

// Shared data between different Pool types
#[derive(Deserialize, Debug)]
struct PoolCommonData {
    pool: PoolCommon,
}
#[derive(Deserialize, Debug)]
struct PoolCommon {
    #[serde(rename = "@type")]
    pool_type: String,
}

// Concentrated Liquidity
#[derive(Deserialize, Debug)]
struct PoolCLData {
    pool: PoolCL,
}
#[derive(Deserialize, Debug)]
struct PoolCL {
    current_sqrt_price: String,
    spread_factor: String,
}

// Default Pool
#[derive(Deserialize, Debug)]
struct PoolDefaultData {
    pool: PoolDefault,
}
#[derive(Deserialize, Debug)]
struct PoolDefault {
    total_weight: String,
    pool_params: PoolParams,
    pool_assets: Vec<PoolAsset>,
}
#[derive(Deserialize, Debug)]
struct PoolParams {
    swap_fee: String,
}
#[derive(Deserialize, Debug)]
struct PoolAsset {
    token: Token,
    weight: String,
}
#[derive(Deserialize, Debug)]
struct Token {
    denom: String,
    amount: String
}


pub async fn fetch_coin_price(pool_id: u64) -> Result<f64, Box<dyn StdError>> {
    // subsitua o {} da url por 1721
    let url = get_osmosis_pool_price_url();
    let url_formated = url.replace("{}", pool_id.to_string().as_str());
    let response = reqwest::get(url_formated).await?;
    let response = response.error_for_status()?;
    
    // Get the raw JSON response
    let raw_json = response.text().await?;

    // Deserialize only the necessary part
    let json_data: PoolCommonData = serde_json::from_str(&raw_json)?;

    println!(">>> Pool ID {}, Type: {}", pool_id, json_data.pool.pool_type);

    match json_data.pool.pool_type.as_str() {
        "/osmosis.concentratedliquidity.v1beta1.Pool" => {
            let json_data: PoolCLData = serde_json::from_str(&raw_json)?;
   
            // Calculate the price based on the sqrt_price
            let sqrt_price: f64 = json_data.pool.current_sqrt_price.parse()
                .map_err(|e| format!("Failed to parse sqrt_price: {}", e))?;
            let mut price = sqrt_price * sqrt_price;
        
            // Apply spread factor
            if let Ok(spread_factor) = json_data.pool.spread_factor.parse::<f64>() {
                price *= 1.0 - spread_factor;
            } else {
                eprintln!("Failed to parse spread factor; using price without discount.");
            }
            
            Ok(price)
        },
        "/osmosis.gamm.v1beta1.Pool" => {
            let json_data: PoolDefaultData = serde_json::from_str(&raw_json)?;
            let pool = json_data.pool;

            let swap_fee: f64 = pool.pool_params.swap_fee.parse()
                .map_err(|e| format!("Failed to parse swap_fee: {}", e))?;

            let asset_0_amount: f64 = pool.pool_assets[0].token.amount.parse()
                .map_err(|e| format!("Failed to parse asset_0_amount: {}", e))?;
            let asset_1_amount: f64 = pool.pool_assets[1].token.amount.parse()
                .map_err(|e| format!("Failed to parse asset_1_amount: {}", e))?;

            let asset_0_weight: f64 = pool.pool_assets[0].weight.parse()
                .map_err(|e| format!("Failed to parse asset_0_weight: {}", e))?;
            let asset_1_weight: f64 = pool.pool_assets[1].weight.parse()
                .map_err(|e| format!("Failed to parse asset_1_weight: {}", e))?;

            // Calculate price assuming equal weight
            let price = (asset_1_amount / asset_1_weight) / (asset_0_amount / asset_0_weight);

            // Apply swap fee
            let price = price * (1.0 - swap_fee);

            Ok(price)
            
        },
        _ => Err(format!("Unknown pool type: {}", json_data.pool.pool_type).into())
    }
    
}

pub fn get_osmosis_rpc_url() -> String {
    env::var("OSMOSIS_STATUS_URL").unwrap_or_else(|_| "https://rpc.osmosis.zone/status".to_string())
}

pub fn get_osmosis_broadcast_tx_url() -> String {
    env::var("OSMOSIS_BROADCAST_TX_URL").unwrap_or_else(|_| "https://lcd-osmosis.zone/cosmos/tx/v1beta1/txs".to_string())
}

pub fn get_osmosis_pool_price_url() -> String {
    env::var("OSMOSIS_POOL_PRICE_URL").unwrap_or_else(|_| "https://lcd-osmosis.imperator.co/osmosis/gamm/v1beta1/pools/{}".to_string())
}

use tokio::time::{sleep, Duration};

async fn fetch_transaction_status(client: &Client, txhash: &str) -> Result<String, Box<dyn StdError>> {
    // Replace with the actual URL for transaction status retrieval
    let url = format!("https://example.com/txs/{}", txhash);

    let response = client.get(&url).send().await?.text().await?;
    Ok(response)
}

async fn monitor_transaction(client: &Client, txhash: &str) {
    let polling_interval = Duration::from_secs(5);
    let timeout_duration = Duration::from_secs(60);
    let start_time = tokio::time::Instant::now();

    loop {
        match fetch_transaction_status(client, txhash).await {
            Ok(status) => {
                println!("Transaction Status: {}", status);
                // Check if the transaction is completed (success or failure)
                if status.contains("completed") || status.contains("failed") {
                    break;
                }
            },
            Err(e) => {
                eprintln!("Error fetching transaction status: {}", e);
                break;
            }
        }

        if tokio::time::Instant::now().duration_since(start_time) >= timeout_duration {
            println!("Timeout: Transaction status check exceeded the maximum duration.");
            break;
        }

        sleep(polling_interval).await;
    }
}