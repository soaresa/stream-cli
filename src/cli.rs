use crate::{constants::get_constants, key_manager::get_account_from_prompt, streamer::Streamer};
use clap::{Parser, Subcommand};
use std::io::{self, Write};
use crate::chains::osmosis::osmosis_key_service::Signer;
use num_format::{Locale, ToFormattedString};
use crate::chains::osmosis::osmosis_account_service::fetch_balances;
use crate::chains::osmosis::osmosis_transaction::summarize_transactions;
use crate::chains::coin::CoinAmount;

/// Stream CLI - Automate your crypto trading strategy
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct TSCli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the stream with specified parameters
    Stream {
        /// Amount in goal per day
        #[arg(short = 'i', long, required_unless_present = "daily_amount_in")]
        daily_amount_out: Option<f64>,

        /// Amount out goal per day
        #[arg(short = 'o', long, required_unless_present = "daily_amount_out")]
        daily_amount_in: Option<f64>,

        /// Streams per day
        #[arg(long)]
        daily_streams: u64,

        /// Target price
        #[arg(short, long)]
        min_price: f64,
    },

    /// Query the balances of an account given an address
    Balance {
        /// The account address to query
        #[arg(short, long)]
        address: String,
    },

    /// Summarize all transactions for all accounts
    Summary,
}

impl TSCli {
    pub async fn run(&self) {
        match &self.command {
            Commands::Stream {
                daily_amount_out,
                daily_amount_in,
                daily_streams,
                min_price,
            } => {
                // Existing logic for starting the stream
                self.run_stream(*daily_amount_out, *daily_amount_in, *daily_streams, *min_price)
                    .await;
            }

            Commands::Balance { address } => {
                // New logic for querying balances
                self.run_balance(address).await;
            }

            Commands::Summary => {
                self.run_summary().await;
            }
        }
    }

    // Method to handle the 'stream' subcommand
    async fn run_stream(
        &self,
        daily_amount_out: Option<f64>,
        daily_amount_in: Option<f64>,
        daily_streams: u64,
        min_price: f64,
    ) {
        // Check if the user has provided valid parameters
        if daily_streams <= 0 || min_price <= 0.0 {
            eprintln!("Invalid parameters provided. Please provide valid values for daily_amount_out, daily_streams, and min_price");
            std::process::exit(0);
        }

        // Get the daily amount out or in based on the user input
        let (swap_type, amount) = if let Some(amount_out) = daily_amount_out {
            ("amount_out", (amount_out * 1_000_000.0) as u64)
        } else if let Some(amount_in) = daily_amount_in {
            ("amount_in", (amount_in * 1_000_000.0) as u64)
        } else {
            unreachable!()
        };

        // Check if the user has provided a valid amount
        if amount <= 0 {
            eprintln!("Invalid amount provided. Please provide a valid value for daily_amount_out or daily_amount_in");
            std::process::exit(0);
        }

        // Get mnemonic from user
        let mnemonic = match get_account_from_prompt("Osmosis") {
            Ok(ret) => ret,
            Err(e) => {
                eprintln!("Error getting account keys: {:?}", e);
                std::process::exit(0);
            }
        };

        // Create signer
        let signer = match Signer::new(&mnemonic) {
            Ok(ret) => ret,
            Err(e) => {
                eprintln!("Error creating signer: {:?}", e);
                std::process::exit(0);
            }
        };

        // Fetch balances
        let balances = match fetch_balances(signer.get_account_address(), None).await {
            Ok(balances) => balances,
            Err(e) => {
                eprintln!("Error fetching account balances: {:?}", e);
                std::process::exit(0);
            }
        };

        // Confirm address and parameters
        if get_user_confirmation(
            &signer.get_account_address(),
            balances,
            amount,
            swap_type,
            daily_streams,
            min_price,
        ) {
            println!("Proceeding...\n");
        } else {
            println!("Exiting...\n");
            std::process::exit(0);
        }

        let streamer = Streamer::new(amount, swap_type, daily_streams, min_price);
        streamer.start(&signer).await;

        println!("Stream service stopped.");
    }

    // Method to handle the 'balance' subcommand
    async fn run_balance(&self, address: &String) {
        // Fetch balances
        let balances = match fetch_balances(&address, None).await {
            Ok(balances) => balances,
            Err(e) => {
                eprintln!("Error fetching account balances: {:?}", e);
                std::process::exit(0);
            }
        };

        // Display balances
        println!("\nBalances for account: {}", address);
        for balance in balances {
            println!("  {}", balance);
        }
    }

    // Method to handle the 'summary' subcommand
    async fn run_summary(&self) {
        match summarize_transactions() {
            Ok(summary) => {
                println!("Transaction Summary:\n{}", serde_json::to_string_pretty(&summary).unwrap());
            }
            Err(e) => {
                eprintln!("Error summarizing transactions: {:?}", e);
            }
        }
    }
}

// Function to get user confirmation (y/n)
fn get_user_confirmation(address: &str, balances: Vec<CoinAmount>, amount: u64, swap_type: &str, daily_streams: u64, min_price: f64) -> bool {   
    // Ask user to confirm the address and params
    println!("\nPlease confirm the following details for the Trade Stream:");
    println!(" 1. Account Address: {}", address);

    // Display account balances
    println!("\n    Account Balances:");
    for balance in &balances {
        println!("    - {}", balance);
    }

    // Display daily amount based on swap type
    match swap_type {
        "amount_out" => {
            let coin_amount = CoinAmount {
                coin: get_constants().token_out,
                amount: amount,
            };
            println!("\n 2. Daily Amount Out: {}", coin_amount);
        },
        "amount_in" => {
            let coin_amount = CoinAmount {
                coin: get_constants().token_out,
                amount: amount,
            };
            println!("\n 2. Daily Amount In:  {}", coin_amount);
        },
        _ => {
            eprintln!("Invalid swap type: {}", swap_type);
            return false;
        }
    }

    // Print additional details
    println!(" 3. Daily Streams:    {}", daily_streams.to_formatted_string(&Locale::en));
    println!(" 4. Min Price:        {} {}", get_constants().token_out, min_price);
    println!(" 5. Token In:         {}", get_constants().token_in);
    println!(" 6. Token Out:        {}", get_constants().token_out);
    println!(" 7. Pool ID:          {}\n", get_constants().pool_id);
    
    print!("Do you want to continue? (y/n): ");
    io::stdout().flush().unwrap(); // Ensures the prompt is displayed correctly

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");

    // Trim whitespace and convert the input to lowercase
    let input = input.trim().to_lowercase();

    // Return true if "y", false if "n", and keep asking if invalid input
    match input.as_str() {
        "y" => true,
        "n" => false,
        _ => {
            println!("Invalid input, please enter 'y' or 'n'");
            get_user_confirmation(address, balances, amount, swap_type, daily_streams, min_price) // Recursively ask again on invalid input
        }
    }
}