pub mod errors;

mod cli;
mod hash;
mod object;
mod store;

use std::path::Path;

use anyhow::Context;
use clap::Parser;

use crate::{
    cli::{Cli, Command},
    object::Object,
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
    let file = file.as_ref();
    let object =
        Object::from_file(file).with_context(|| format!("failed to read '{}'", file.display()))?;

    let store = ObjectStore::new(Path::new("."));

    store.put(&object)?;

    println!("{}", object.id());

    Ok(())
}
