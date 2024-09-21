use crate::{key_manager::get_account_from_prompt, streamer::Streamer};
use clap::Parser;
use std::io::{self, Write};
use crate::chains::osmosis::osmosis_key_service::Signer;
use num_format::{Locale, ToFormattedString};

/// start the stream with some constants
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct TSCli {
    /// dollar goal per day
    #[arg(short, long)]
    pub daily_amount_out: u64,
    
    /// streams per day
    #[arg(long)]
    pub daily_streams: u64,

    /// target price
    #[arg(short, long)]
    pub min_price: f64,
}

impl TSCli {
    pub async fn run(&self) {
        println!("Starting stream");

        // get mnemonic from user
        let mnemonic = match get_account_from_prompt("Osmosis") {
            Ok(ret) => ret,
            Err(e) => {
                eprintln!("Error getting account keys: {:?}", e);
                std::process::exit(0);
            }
        };

        // create signer
        let signer = match Signer::new(&mnemonic) {
            Ok(ret) => ret,
            Err(e) => {
                eprintln!("Error creating signer: {:?}", e);
                std::process::exit(0);
            }
        };
               
        // confirm address and parameters
        if get_user_confirmation(
            &signer.get_account_address(),
            self.daily_amount_out,
            self.daily_streams,
            self.min_price,
        ) {
            println!("Proceeding...");
        } else {
            println!("Exiting...");
            std::process::exit(0);
        }

        let streamer = Streamer::new(self.daily_amount_out, self.daily_streams, self.min_price);
        streamer.start(&signer).await;
    }
}

// Function to get user confirmation (y/n)
fn get_user_confirmation(address: &str, daily_dollar: u64, daily_streams: u64, min_price: f64) -> bool {
    // ask user to confirm the address and params
    println!("Please confirm the following details:");
    println!(" - Address: {}", address);
    println!(" - Daily Dollar: ${}", daily_dollar.to_formatted_string(&Locale::en));
    println!(" - Daily Streams: {}", daily_streams.to_formatted_string(&Locale::en));
    println!(" - Min Price: ${}", min_price);
    
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
            get_user_confirmation(address, daily_dollar, daily_streams, min_price) // Recursively ask again on invalid input
        }
    }
}