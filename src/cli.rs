use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    /// Path to the file to process.
    pub file: PathBuf,
}
