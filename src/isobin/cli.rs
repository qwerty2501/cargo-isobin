use clap::{Parser, Subcommand};
use std::path::PathBuf;
#[derive(Parser)]
#[clap(author, version, about)]
pub struct Arguments {
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
