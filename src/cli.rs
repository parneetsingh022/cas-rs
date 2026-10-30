use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    /// Path to the file to process.
    pub file: PathBuf,
}
