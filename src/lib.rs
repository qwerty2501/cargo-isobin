mod config;
mod errors;

pub use errors::*;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[cfg(test)]
use rstest::*;

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Args {
    /// Sets a custom config file
    #[clap(short, long, parse(from_os_str), value_name = "FILE")]
    config: Option<PathBuf>,
    #[clap(subcommand)]
    subcommand: Option<SubCommands>,
}

#[derive(Subcommand)]
pub enum SubCommands {
    Install {},
}

#[derive(Default)]
pub struct App;

impl App {
    pub fn run(&self, _args: Args) -> Result<()> {
        Err(Errors::Test().into())
    }
}
