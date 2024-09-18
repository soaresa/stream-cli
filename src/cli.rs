use crate::{poll, stream};
use clap::Parser;
/// start the stream with some constants
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct TStreamCli {
    /// dollar goal per day
    #[arg(short, long)]
    pub daily_dollar: u64,

    /// percent range
    #[arg(short, long)]
    pub units: Option<f64>,

    /// target price
    #[arg(short, long, requires("units"))]
    pub price: Option<f64>,
}

impl TStreamCli {
    pub fn run(&self) {
        let (tx, rx) = stream::init();
        // start the trade service
        let trade_service_handle = stream::listen(rx);

        // start polling service
        // does not exit, just continues on a loop.
        let _unused_handle = poll::start_polling(tx);

        // join() keeps thread alive
        println!("Stream started, exit with ctrl+c");
        trade_service_handle.join().expect("could not complete tasks in stream");
    }
}
