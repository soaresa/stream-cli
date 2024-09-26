use tokio::time::{sleep, Duration};
use chrono::{DateTime, Utc, Duration as ChronoDuration};
use rand::Rng;
use crate::chains::osmosis::osmosis_key_service::Signer;
use crate::trade_service::TradeTask;
use crate::config::constants::get_constants;
use std::io::{self, Write};
use tokio::sync::watch;
use log::{info, warn, error};

const POLL_INTERVAL: u64 = 1000; // in milliseconds

pub async fn start_polling(
    signer: &Signer,
    daily_amount: u64,
    swap_type: &'static str,
    streams_per_day: u64,
    min_price: f64,
) {
    info!("Starting the polling service...");
    // Initializations
    let mut end_window_time: DateTime<Utc> = Utc::now();
    let mut next_trade: DateTime<Utc> = Utc::now();
    let mut trade_executed = true;
    let trade_amount: u64 = daily_amount / streams_per_day;
    let constants = get_constants();
    let mut jump = false;

    // Use watch channel to signal stop request
    let (tx, rx) = watch::channel(false);

    // Create a task to listen for Ctrl+C in a separate async block
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("failed to listen for Ctrl+C");
        println!("\n\nCtrl+C pressed. Stopping the service...\n\n");
        let _ = tx.send(true); // Set the stop flag
    });
    
    loop {
        // Check for stop request
        if *rx.borrow() {
            println!("\n\n<<< Stopping the service gracefully >>>\n");
            break;
        }

        // Sleep asynchronously
        sleep(Duration::from_millis(POLL_INTERVAL)).await;

        // 1. Check if we need a new trade window
        let now = Utc::now();
        if end_window_time < now {
            if !trade_executed {
                warn!("Trade not executed in the last window. Skipping the next window.");
            }

            if jump { 
                println!("");
                jump = false;
            }

            trade_executed = false;

            // 1.1. Calculate the end time of the next window
            let window_duration = ChronoDuration::hours(24) / streams_per_day as i32;
            end_window_time = now + window_duration;

            // 1.2. Generate a random time between now and the end of the window
            next_trade = generate_next_trade_time(now, end_window_time);
        }

        // 2. Check if we have already traded in this window
        if trade_executed {
            let diff = end_window_time - now;
            let remaining = format!("{:02}:{:02}:{:02}", diff.num_hours(), diff.num_minutes() % 60, diff.num_seconds() % 60);
            print!("\rNext window starts in: {}", remaining);
            io::stdout().flush().unwrap();
            jump = true;
            continue;
        }

        // 3. Check if it's time to trade
        if next_trade < now {
            if jump { 
                println!("");
                jump = false;
            };
            println!("Try to execute trade...");
            
            // Create a new trade task
            let task = TradeTask::new(
                constants.pool_id,
                constants.token_in.clone(),
                constants.token_out.clone(),
                trade_amount,
                swap_type,
                min_price,
            );

            // Execute the task directly
            let ret = task.execute(signer).await;

            // print response
            match ret {
                Ok(boo) => {
                    trade_executed = boo;
                    if boo {
                        println!("Trade executed with success\n");
                    } else {
                        println!("Trade not executed");
                    }
                },
                Err(e) => {
                    error!("Error executing trade: {:?}", e);
                }
            }
            
            continue;
        }

        let diff = next_trade - now;
        let remaining = format!("{:02}:{:02}:{:02}", diff.num_hours(), diff.num_minutes() % 60, diff.num_seconds() % 60);
        print!("\rNext trade starts in: {}", remaining);
        jump = true;
        io::stdout().flush().unwrap();
    }
}

fn generate_next_trade_time(now: DateTime<Utc>, end_window_time: DateTime<Utc>) -> DateTime<Utc> {
    let now_timestamp = now.timestamp();
    let end_timestamp = end_window_time.timestamp();
    let mut rng = rand::thread_rng();
    let random_timestamp = rng.gen_range(now_timestamp..end_timestamp);

    // Use DateTime::from_timestamp to create the DateTime directly
    DateTime::<Utc>::from_timestamp(random_timestamp, 0).unwrap()
}


