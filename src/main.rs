use clap::Parser;
use tstream::cli;

fn main() {
    let app = cli::TStreamCli::parse();

    app.run()
}
