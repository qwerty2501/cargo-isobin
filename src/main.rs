mod config;
mod errors;

use errors::*;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[cfg(test)]
use rstest::*;

#[cfg(test)]
use pretty_assertions::{assert_eq, assert_ne};

#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    /// Sets a custom config file
    #[clap(short, long, parse(from_os_str), value_name = "FILE")]
    config: Option<PathBuf>,
    #[clap(subcommand)]
    subcommand: Option<SubCommands>,
}

#[derive(Subcommand)]
enum SubCommands {
    Install {},
}

struct App;

impl App {
    fn new() -> Self {
        Self {}
    }
    fn run(&self, args: Args) -> Result<()> {
        Err(Errors::Test().into())
    }
}

fn main() -> Result<()> {
    let app = App::new();

    let args = Args::parse();
    app.run(args)
}
