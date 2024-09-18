use crate::stream;
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
    stream::start_stream();
  }
}
