use crate::{constants::get_constants, key_manager::get_account_from_prompt, streamer::Streamer};
use clap::{Parser, Subcommand};
use std::io::{self, Write};
use crate::chains::osmosis::osmosis_key_service::Signer;
use num_format::{Locale, ToFormattedString};
use crate::chains::osmosis::osmosis_account_service::fetch_balances;
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
        daily_amount_out: Option<u64>,

        /// Amount out goal per day
        #[arg(short = 'o', long, required_unless_present = "daily_amount_out")]
        daily_amount_in: Option<u64>,

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
        }
    }

    // Method to handle the 'stream' subcommand
    async fn run_stream(
        &self,
        daily_amount_out: Option<u64>,
        daily_amount_in: Option<u64>,
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
            ("amount_out", amount_out)
        } else if let Some(amount_in) = daily_amount_in {
            ("amount_in", amount_in)
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
            daily_amount_out,
            daily_amount_in,
            daily_streams,
            min_price,
        ) {
            println!("Proceeding...");
        } else {
            println!("Exiting...");
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
        println!("Balances for account: {}", address);
        for balance in balances {
            println!(
                "- {} {}",
                balance.coin,
                balance.amount.to_formatted_string(&Locale::en)
            );
        }
    }
}

// Function to get user confirmation (y/n)
fn get_user_confirmation(address: &str, balances: Vec<CoinAmount>, daily_amount_out: Option<u64>, daily_amount_in: Option<u64>, daily_streams: u64, min_price: f64) -> bool {
    // ask user to confirm the address and params
    println!("\nPlease confirm the following details for the Trade Stream:");
    println!(" 1. Account Address: {}", address);
    for balance in &balances {
        println!("    - {} {}", balance.coin, balance.amount.to_formatted_string(&Locale::en));
    }
    if let Some(amount_out) = daily_amount_out {
        println!(" 2. Daily Amount Out: {} {}", get_constants().token_out, amount_out.to_formatted_string(&Locale::en));
    }
    if let Some(amount_in) = daily_amount_in {
        println!(" 2. Daily Amount In: {} {}", get_constants().token_in, amount_in.to_formatted_string(&Locale::en));
    }
    println!(" 3. Daily Streams: {}", daily_streams.to_formatted_string(&Locale::en));
    println!(" 4. Min Price: {} {}", get_constants().token_out, min_price);
    println!(" 5. Token In: {}", get_constants().token_in);
    println!(" 6. Pool ID: {}\n", get_constants().pool_id);
    
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
            get_user_confirmation(address, balances, daily_amount_out, daily_amount_in, daily_streams, min_price) // Recursively ask again on invalid input
        }
    }
}