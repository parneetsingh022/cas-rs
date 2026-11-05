use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    /// Provide a custom storage directory path
    #[arg(long, short, global = true)]
    pub store: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Save a file to the store and output its unique hash.
    Put { file: Option<PathBuf> },

}
