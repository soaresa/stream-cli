use tokio::time::{sleep, Duration};
use chrono::{DateTime, Utc, Duration as ChronoDuration};
use rand::Rng;
use crate::chains::osmosis::osmosis_key_service::Signer;
use crate::trade_service::TradeTask;
use crate::constants::get_constants;
use num_format::{Locale, ToFormattedString};

const POLL_INTERVAL: u64 = 1000; // in milliseconds

pub async fn start_polling(
    signer: &Signer,
    daily_amount_out: u64,
    streams_per_day: u64,
    min_price: f64,
) {
    // Initializations
    let mut end_window_time: DateTime<Utc> = Utc::now();
    let mut next_trade: DateTime<Utc> = Utc::now();
    let mut trade_executed = false;
    let amount_out: u64 = daily_amount_out / streams_per_day;
    let constants = get_constants();

    loop {
        // Sleep asynchronously
        sleep(Duration::from_millis(POLL_INTERVAL)).await;

        // 1. Check if we need a new trade window
        let now = Utc::now();
        if end_window_time < now {
            trade_executed = false;

            // 1.1. Calculate the end time of the next window
            let window_duration = ChronoDuration::hours(24) / streams_per_day as i32;
            end_window_time = now + window_duration;

            // 1.2. Generate a random time between now and the end of the window
            next_trade = generate_next_trade_time(now, end_window_time);
            println!(">>> Next trade scheduled for {}", next_trade.format("%Y-%m-%d %H:%M:%S"));
        }

        // 2. Check if we have already traded in this window
        if trade_executed {
            continue;
        }

        // 3. Check if it's time to trade
        if next_trade < now {
            // Create a new trade task
            let task = TradeTask::new(
                constants.pool_id,
                constants.token_in.clone(),
                constants.token_out.clone(),
                amount_out,
                min_price,
            );

            // Execute the task directly
            trade_executed = task.execute(signer).await;
            println!(
                "$$$ Trade executed {} at {} for amount {}",
                trade_executed,
                now.format("%Y-%m-%d %H:%M:%S"),
                amount_out.to_formatted_string(&Locale::en)
            );
        }
    }
}

fn generate_next_trade_time(now: DateTime<Utc>, end_window_time: DateTime<Utc>) -> DateTime<Utc> {
    let now_timestamp = now.timestamp();
    let end_timestamp = end_window_time.timestamp();
    let mut rng = rand::thread_rng();
    let random_timestamp = rng.gen_range(now_timestamp..end_timestamp);
    DateTime::<Utc>::from_utc(
        chrono::NaiveDateTime::from_timestamp(random_timestamp, 0),
        Utc,
    )
}
