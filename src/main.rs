use clap::Parser;
use tstream::cli;

fn main() {
    let _log2 = log2::open("ts.log")
      .size(100 * 1024 * 1024)
      .rotate(20)
      .tee(true)
      .module(true)
      .start();

    cli::TSCli::parse().run();
}
