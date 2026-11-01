use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::{NamedTempFile, tempdir};

const BIN_NAME: &str = env!("CARGO_PKG_NAME");

#[test]
fn test_put_prints_hash_to_stdout() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = tempdir()?;

    let mut file = NamedTempFile::new()?;
    write!(file, "test data")?;

    let expected_hash = "916f0027a575074ce72a331777c3478d6513f786a591bd892da1a577bf2335f9";

    let mut cmd = Command::cargo_bin(BIN_NAME)?;

    // Force the command to execute inside the temp workspace
    cmd.current_dir(workspace.path())
        .arg("put")
        .arg(file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::starts_with(expected_hash));

    Ok(())
}

#[test]
fn test_put_creates_object_file_in_store() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = tempdir()?;

    let mut file = NamedTempFile::new()?;
    write!(file, "test data")?;

    let expected_hash = "916f0027a575074ce72a331777c3478d6513f786a591bd892da1a577bf2335f9";

    let mut cmd = Command::cargo_bin(BIN_NAME)?;

    cmd.current_dir(workspace.path())
        .arg("put")
        .arg(file.path());

    cmd.assert().success();

    let (dir, file_name) = expected_hash.split_at(2);

    // Update this chain to explicitly include .cas and objects
    let stored_file_path = workspace
        .path()
        .join(".cas")
        .join("objects")
        .join(dir)
        .join(file_name);

    assert!(
        stored_file_path.exists(),
        "Expected object file not found at {:?}",
        stored_file_path
    );

    file.close()?;
    workspace.close()?;

    Ok(())
}
