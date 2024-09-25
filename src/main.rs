use env_logger;
use clap::Parser;
use tstream::cli;
use tstream::configs;
use tokio::runtime::Runtime;

fn main() {
    env_logger::init();

    configs::initialize();

    let rt = Runtime::new().unwrap();
    rt.block_on(cli::TSCli::parse().run());
}
