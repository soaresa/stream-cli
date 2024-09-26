use serde::Deserialize;
use std::error::Error as StdError;
use reqwest;
use crate::config::CONFIG;
use crate::chains::coin::Coin;
use crate::chains::chain::ChainType;
use crate::chains::osmosis::osmosis_key_service::Signer;
use crate::chains::osmosis::osmosis_account_service::fetch_account_info;
use super::osmosis_transaction::broadcast_tx;

use osmosis_std::types::osmosis::gamm::v1beta1::{MsgSwapExactAmountOut, SwapAmountOutRoute, MsgSwapExactAmountIn, SwapAmountInRoute};
use osmosis_std::types::cosmos::base::v1beta1::Coin as OsmosisCoin;

use serde_json::json;
use cosmrs::proto::cosmos::tx::v1beta1::{SimulateRequest, SimulateResponse};

use cosmrs::tendermint::{block::Height, chain::Id};
use cosmrs::tx::{Body, Fee, AuthInfo, SignDoc, Tx};
use cosmrs::Any;
use cosmrs::Coin as CosmosCoin;
use cosmrs::Decimal;

use anyhow::Result;
use prost::Message;

use reqwest::Client;

// TODO: WIP Function to simulate a transaction cost
pub async fn simulate_tx(tx: Tx) -> Result<()> {
    // Step 1: Encode the transaction into the protobuf format
    let proto_tx: cosmrs::proto::cosmos::tx::v1beta1::Tx = tx.into();
    let mut tx_bytes = Vec::new();
    proto_tx.encode(&mut tx_bytes).map_err(|e| anyhow::anyhow!("Failed to encode Tx: {}", e))?;

    // Step 2: Convert the tx bytes to Base64
    let tx_base64 = base64::encode(&tx_bytes);

    // Step 3: Prepare the request body
    let simulate_body = json!({
        "tx_bytes": tx_base64
    });

    // Step 4: Make the request to the simulate endpoint
    let client = Client::new();
    let simulate_url = "https://lcd.osmotest5.osmosis.zone/cosmos/tx/v1beta1/simulate".to_string();
    let response = client
        .post(simulate_url)
        .json(&simulate_body)
        .send()
        .await?;

    // Step 5: Parse the response
    let response_json = response.json::<serde_json::Value>().await.map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e))?;

    // Step 6: Extract relevant data
    if let Some(simulate_response) = response_json.get("gas_info") {
        let gas_used = simulate_response["gas_used"].as_str();
        let gas_wanted = simulate_response["gas_wanted"].as_str();

        println!("Gas Used: {:?}, Gas Wanted: {:?}", gas_used, gas_wanted);
    } else {
        println!("Failed to get gas info from response: {:?}", response_json);
    }

    Ok(())
}

pub async fn perform_swap(
    signer: &Signer,
    pool_id: u64,
    coin_in: Coin,
    coin_out: Coin,    
    amount: u64,
    swap_type: &str,
    min_price: f64,
) -> Result<bool, anyhow::Error> {
    
    // Step 1. Get the sender address
    let sender_address = signer.get_account_address();

    // Step 2. Create the swap message
    let msg_swap = match swap_type {
        "amount_out" => create_msg_swap_exact_amount_out(sender_address, pool_id, coin_in, coin_out, amount, min_price),
        "amount_in" => create_msg_swap_exact_amount_in(sender_address, pool_id, coin_in, coin_out, amount, min_price),
        _ => Err(anyhow::anyhow!("Invalid swap type: {}", swap_type)),
    }?;

    // Step 3. Get the current block height
    let current_height = get_current_block_height().await.map_err(|e| anyhow::anyhow!("Failed to get current block height: {}", e))?;
    let timeout_height = current_height + 200;  // Set a future timeout height

    // Step 4. Create TxBody
    let tx_body = Body {
        messages: vec![msg_swap],
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
        amount: Decimal::from(CONFIG.gas_config.amount),
    }, CONFIG.gas_config.gas_limit);
    let auth_info = AuthInfo {
        signer_infos: vec![signer_info],
        fee,
    };  

    // Step 7: Create and sign the doc
    let comos_id = ChainType::Osmosis.chain_id();
    let chain_id = Id::try_from(comos_id.clone())?;
    let sign_doc = SignDoc::new(&tx_body, &auth_info, &chain_id, account_number).map_err(|e| anyhow::anyhow!("Failed to create SignDoc: {}", e))?;
    let tx_bytes = signer.sign_doc(sign_doc).map_err(|e| anyhow::anyhow!("Failed to sign the transaction: {}", e))?;

    // Step 8: Create and broadcast the transaction
    let tx_parsed = Tx::from_bytes(&tx_bytes).map_err(|e| anyhow::anyhow!("Failed to parse transaction bytes: {}", e))?;
    // simulate_tx(tx_parsed.clone()).await?;
    broadcast_tx(tx_parsed, sender_address, pool_id, coin_in, coin_out, amount, swap_type, min_price).await
}

fn create_msg_swap_exact_amount_out(sender_address: &str, pool_id: u64, coin_in: Coin, coin_out: Coin, amount: u64, min_price: f64) -> Result<Any> {
    // Step 1. Calc max token in amount
    let token_in_max_amount: u64 = (amount as f64 / min_price) as u64;

    // Step 2. Create swap message
    let msg_swap = MsgSwapExactAmountOut {
        sender: sender_address.to_string(),
        routes: vec![SwapAmountOutRoute {
            pool_id,
            token_in_denom: coin_in.denom().parse().map_err(|e| anyhow::anyhow!("Failed to parse coin denom: {}", e))?,
        }],
        token_out: Some(OsmosisCoin {
            denom: coin_out.denom().parse().map_err(|e| anyhow::anyhow!("Failed to parse coin denom: {}", e))?,
            amount: amount.to_string(),
        }),
        token_in_max_amount: token_in_max_amount.to_string(),
    };

    Ok(Any {
        type_url: "/osmosis.gamm.v1beta1.MsgSwapExactAmountOut".to_string(),
        value: msg_swap.encode_to_vec(),
    })
}

fn create_msg_swap_exact_amount_in(sender_address: &str, pool_id: u64, coin_in: Coin, coin_out: Coin, amount: u64, min_price: f64) -> Result<Any> {
    // Step 1. Calc min token out amount
    let token_out_min_amount: u64 = (amount as f64 * min_price) as u64;

    // Step 2. Create swap message
    let msg_swap = MsgSwapExactAmountIn {
        sender: sender_address.to_string(),
        routes: vec![SwapAmountInRoute {
            pool_id,
            token_out_denom: coin_out.denom().parse().map_err(|e| anyhow::anyhow!("Failed to parse coin denom: {}", e))?,
        }],
        token_in: Some(OsmosisCoin {
            denom: coin_in.denom().parse().map_err(|e| anyhow::anyhow!("Failed to parse coin denom: {}", e))?,
            amount: amount.to_string(),
        }),
        token_out_min_amount: token_out_min_amount.to_string(),
    };

    Ok(Any {
        type_url: "/osmosis.gamm.v1beta1.MsgSwapExactAmountIn".to_string(),
        value: msg_swap.encode_to_vec(),
    })
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
    //denom: String,
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

    // Parse the pool type and get the price
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

fn get_osmosis_rpc_url() -> String {
    CONFIG.osmosis_status_url.clone()
}

fn get_osmosis_pool_price_url() -> String {
    CONFIG.osmosis_pool_price_url.clone()
}