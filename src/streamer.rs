use crate::{poll_service, trade_service};
// use log::{info, error};
use log2::*;
/// creates a Streamer struct, which will enclose the services
/// needed to run the trade stream
// main app logic, and entry point for external libraries
pub struct Streamer {
    /// dollar goal per day
    pub daily_dollar: u64,

    /// units
    pub units: Option<f64>,

    /// target price
    pub price: Option<f64>,
}

impl Streamer {
    pub fn new(daily_dollar: u64, units: Option<f64>, price: Option<f64>) -> Self {
        Streamer {
            daily_dollar,
            units,
            price,
        }
    }

    // TODO: determine keys type
    pub fn start(&self, keys: String) {
        let _log2 = log2::open("ts.log")
            .size(100 * 1024 * 1024)
            .rotate(20)
            .tee(true)
            .module(true)
            .start();

        trace!("send order request to server");
        debug!("receive order response");
        info!("order was executed");
        warn!("network speed is slow");
        error!("network connection was broken");

        println!("trade stream started");
        info!("using keys: {}", keys);

        let (tx, rx) = trade_service::init();
        // start the trade service
        let trade_service_handle = trade_service::listen(rx);

        // start polling service
        // does not exit, just continues on a loop.
        let _unused_handle = poll_service::start_polling(tx);

        // join() keeps thread alive
        println!("Stream started, exit with ctrl+c");
        trade_service_handle
            .join()
            .expect("could not complete tasks in stream");
    }
}
