pub mod errors;

mod cli;
mod hash;
mod object;
mod store;

use std::{fs::File, path::Path};

use anyhow::Context;
use clap::Parser;

use crate::{
    cli::{Cli, Command},
    store::ObjectStore,
};

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    match args.command {
        Command::Put { file } => handle_put(file)?,
    }

    Ok(())
}

fn handle_put(file: impl AsRef<Path>) -> anyhow::Result<()> {
    let file_path = file.as_ref();
    let file_stream = File::open(file_path)
        .with_context(|| format!("failed to open '{}'", file_path.display()))?;

    let store = ObjectStore::new(Path::new("."))?;
    let object = store
        .put(file_stream)
        .with_context(|| format!("failed to store '{}'", file_path.display()))?;

    eprintln!(
        "Stored {:?} object ({} bytes) from '{}'",
        object.kind(),
        object.size(),
        file_path.display()
    );

    println!("{}", object.id());

    Ok(())
}
