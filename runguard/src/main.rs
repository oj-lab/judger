mod cli;

use clap::Parser;
use cli::Cli;

fn main() {
    let _ = Cli::parse();
}
