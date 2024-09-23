use clap::Parser;
use tstream::cli;
use tstream::configs;
// use tokio::runtime::Runtime;

#[tokio::main]
async fn main() {
    let _log2 = log2::open("ts.log")
      .size(100 * 1024 * 1024)
      .rotate(20)
      .tee(true)
      .module(true)
      .start();

    configs::initialize();

    // let rt = Runtime::new().unwrap();
    // rt.block_on(cli::TSCli::parse().run());
    cli::TSCli::parse().run().await;
}
