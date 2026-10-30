mod cli;
mod hash;
mod object;
mod store;

use std::path::Path;

use anyhow::Context;
use clap::Parser;
use cli::Cli;
use object::Object;
use store::ObjectStore;

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let object = Object::from_file(&args.file)
        .with_context(|| format!("failed to read '{}'", args.file.display()))?;

    let store = ObjectStore::new(Path::new("."));

    store.put(&object)?;

    println!("{}", object.id());

    Ok(())
}
