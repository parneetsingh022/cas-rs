pub mod errors;

mod cli;
mod hash;
mod object;
mod store;

use std::path::Path;

use anyhow::Context;
use clap::Parser;

use crate::{cli::Cli, object::Object, store::ObjectStore};

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let object = Object::from_file(&args.file)
        .with_context(|| format!("failed to read '{}'", args.file.display()))?;

    let store = ObjectStore::new(Path::new("invalid/path"));

    store.put(&object)?;

    println!("{}", object.id());

    Ok(())
}
