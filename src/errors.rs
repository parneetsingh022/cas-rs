use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),

    #[error("object '{0}' not found in store")]
    NotFound(String),

    #[error("no such file or directory: '{}'", path.display())]
    InvalidPath { path: PathBuf },

    #[error("invalid object id hex format: {0}")]
    HexParse(#[from] hex::FromHexError),
}
