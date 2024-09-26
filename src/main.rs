use env_logger;
use clap::Parser;
use tstream::cli;
use tstream::config::env_config;
use tokio::runtime::Runtime;

fn main() {
    env_logger::init();

    env_config::initialize();

    let rt = Runtime::new().unwrap();
    rt.block_on(cli::TSCli::parse().run());
}
