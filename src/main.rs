mod cli;
mod hash;
mod object;
mod store;

use std::{io, path::Path};

use clap::Parser;
use cli::Cli;
use object::Object;
use store::ObjectStore;

fn main() -> io::Result<()> {
    let args = Cli::parse();

    let object = Object::from_file(&args.file)?;
    let store = ObjectStore::new(Path::new("."));

    store.put(&object)?;

    println!("{}", object.id());

    Ok(())
}
