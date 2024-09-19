use clap::Parser;
use tstream::cli;

fn main() {
    cli::TSCli::parse().run();
}
