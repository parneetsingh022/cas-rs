use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error occurred: {0}")]
    Io(#[from] std::io::Error),

    #[error("Object not found: {0}")]
    NotFound(String),

    #[error("Invalid object path: {path}")]
    InvalidPath { path: PathBuf },
}
