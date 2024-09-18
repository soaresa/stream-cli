use tstream::cli;
use clap::Parser;

fn main() {
  let app = cli::TStreamCli::parse();

  app.run()
}
