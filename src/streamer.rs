use crate::poll_service;
use crate::chains::osmosis::osmosis_key_service::Signer;
use log::info;
// use tokio::task::LocalSet;

/// creates a Streamer struct, which will enclose the services
/// needed to run the trade stream
// main app logic, and entry point for external libraries
pub struct Streamer {
    /// amount goal per day
    pub daily_amount_out: u64,

    /// streams per day
    pub daily_streams: u64,

    /// min price
    pub min_price: f64,
}

impl Streamer {
    pub fn new(daily_amount_out: u64, daily_streams: u64, min_price: f64) -> Self {
        Streamer {
            daily_amount_out: daily_amount_out * 1_000_000,
            daily_streams,
            min_price
        }
    }

    pub async fn start(&self, signer: &Signer) {
        info!("Using account: {}", signer.get_account_address());

        // Create a LocalSet to run !Send futures on the current thread
        // let local = LocalSet::new();

        // Clone necessary values to move into the async tasks
        let daily_amount_out = self.daily_amount_out;
        let daily_streams = self.daily_streams;
        let min_price = self.min_price;

        // Since we cannot clone `signer`, we need to ensure that it's used within the same scope

        // Start the polling service
        // local.run_until(async move {
            poll_service::start_polling(
                signer,
                daily_amount_out,
                daily_streams,
                min_price,
            )
            .await;
        // })
        // .await;

        // No need to spawn additional tasks that require `signer`
        // Any additional tasks that need `signer` should be run within this scope
    }
}
