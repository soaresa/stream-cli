use crate::{key_manager::get_account_from_prompt, streamer::Streamer};
use clap::Parser;
/// start the stream with some constants
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct TSCli {
    /// dollar goal per day
    #[arg(short, long)]
    pub daily_dollar: u64,

    /// units
    #[arg(short, long)]
    pub units: Option<f64>,

    /// target price
    #[arg(short, long, requires("units"))]
    pub price: Option<f64>,
}

impl TSCli {
    pub fn run(&self) {
        let keys = get_account_from_prompt("osmo").expect("could not get keys from prompt");

        let s = Streamer::new(self.daily_dollar, self.units, self.price);
        s.start(keys);
    }
}
