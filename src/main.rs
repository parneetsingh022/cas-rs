pub mod errors;

mod cli;
mod hash;
mod object;
mod store;

use std::{
    fs::File,
    io::{self, IsTerminal},
    path::Path,
};

use anyhow::Context;
use clap::Parser;

use crate::{
    cli::{Cli, Command},
    store::ObjectStore,
};

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let store_path = args.store;

    match args.command {
        Command::Put { file } => handle_put(file, store_path)?,
    }

    Ok(())
}

fn handle_put(
    file: Option<impl AsRef<Path>>,
    store_path: Option<impl AsRef<Path>>,
) -> anyhow::Result<()> {
    let file_path = file.as_ref().map(AsRef::as_ref);

    let store = ObjectStore::new(store_path.as_ref())?;

    let object = match file_path {
        Some(path) => {
            let file =
                File::open(path).with_context(|| format!("failed to open '{}'", path.display()))?;

            store
                .put(file)
                .with_context(|| format!("failed to store '{}'", path.display()))?
        }
        None => {
            if io::stdin().is_terminal() {
                anyhow::bail!("no input file specified and stdin is a terminal");
            }

            store
                .put(io::stdin().lock())
                .context("failed to store object from stdin")?
        }
    };

    eprintln!(
        "Stored {:?} object ({} bytes) from '{}'",
        object.kind(),
        object.size(),
        file_path
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "stdin".to_string())
    );

    println!("{}", object.id());

    Ok(())
}
