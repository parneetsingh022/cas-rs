use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::{NamedTempFile, tempdir};

const BIN_NAME: &str = env!("CARGO_PKG_NAME");

#[test]
fn test_put_prints_hash_to_stdout() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = tempdir()?;

    let mut file = NamedTempFile::new()?;
    write!(file, "test put prints hash to stdout")?;

    let expected_hash = "029d420eff8ca02f0d4b5b2934bb2941461cdd652bf032c92069e93706fc038e";

    let mut cmd = Command::cargo_bin(BIN_NAME)?;

    // Isolate storage to prevent modifying the user's default storage.
    cmd.current_dir(workspace.path())
        .env("HASHB_DEFAULT_STORAGE", workspace.path())
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
    write!(file, "test put object file in store")?;

    let expected_hash = "9797794c0dfb7457dd0be3e9b0b5fd85baa393b192db93b7b423f4fd3d21ae8e";

    let mut cmd = Command::cargo_bin(BIN_NAME)?;

    cmd.current_dir(workspace.path())
        .env("HASHB_DEFAULT_STORAGE", workspace.path())
        .arg("put")
        .arg(file.path());

    cmd.assert().success();

    let (dir, file_name) = expected_hash.split_at(2);

    // Update this chain to explicitly include .hsb and objects
    let stored_file_path = workspace.path().join("objects").join(dir).join(file_name);

    assert!(
        stored_file_path.exists(),
        "Expected object file not found at {:?}",
        stored_file_path
    );

    file.close()?;
    workspace.close()?;

    Ok(())
}

#[test]
fn test_put_reads_from_stdin() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = tempdir()?;

    let input = b"test put reads from stdin";
    let expected_hash = blake3::hash(input).to_hex().to_string();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;

    cmd.current_dir(workspace.path())
        .env("HASHB_DEFAULT_STORAGE", workspace.path())
        .arg("put")
        .write_stdin(input.as_slice());

    cmd.assert()
        .success()
        .stdout(predicate::str::starts_with(expected_hash.as_str()));

    let (dir, file_name) = expected_hash.split_at(2);

    let stored_file_path = workspace.path().join("objects").join(dir).join(file_name);

    assert!(
        stored_file_path.exists(),
        "Expected object file not found at {:?}",
        stored_file_path
    );

    Ok(())
}

#[test]
fn test_put_with_custom_store() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = tempdir()?;
    let store_path = workspace.path().join("custom_store");

    // Custom storage directory must exist before use.
    std::fs::create_dir(&store_path)?;

    let mut file = NamedTempFile::new()?;
    write!(file, "test put with custom store")?;

    let expected_hash = blake3::hash(b"test put with custom store")
        .to_hex()
        .to_string();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;

    cmd.current_dir(workspace.path())
        .env("HASHB_DEFAULT_STORAGE", workspace.path())
        .arg("--store")
        .arg(&store_path)
        .arg("put")
        .arg(file.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::starts_with(expected_hash.as_str()));

    let (dir, file_name) = expected_hash.split_at(2);

    let stored_file_path = store_path.join("objects").join(dir).join(file_name);

    assert!(stored_file_path.is_file());

    Ok(())
}

#[test]
fn test_store_flag_overrides_env() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = tempdir()?;

    let default_store = workspace.path().join("default_store");
    let custom_store = workspace.path().join("custom_store");

    // Explicit custom storage directory must exist.
    std::fs::create_dir(&custom_store)?;

    let input = b"test store flag overrides environment";
    let expected_hash = blake3::hash(input).to_hex().to_string();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;

    cmd.current_dir(workspace.path())
        .env("HASHB_DEFAULT_STORAGE", &default_store)
        .arg("--store")
        .arg(&custom_store)
        .arg("put")
        .write_stdin(input.as_slice());

    cmd.assert().success();

    let (dir, file_name) = expected_hash.split_at(2);

    let custom_object = custom_store.join("objects").join(dir).join(file_name);
    let default_object = default_store.join("objects").join(dir).join(file_name);

    assert!(custom_object.is_file());
    assert!(!default_object.exists());
    assert!(!default_store.exists());

    Ok(())
}

#[test]
fn test_put_stdin_with_custom_store() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = tempdir()?;
    let store_path = workspace.path().join("custom_store");

    // Custom storage directory must exist before use.
    std::fs::create_dir(&store_path)?;

    let input = b"test stdin with custom store";
    let expected_hash = blake3::hash(input).to_hex().to_string();

    let mut cmd = Command::cargo_bin(BIN_NAME)?;

    cmd.current_dir(workspace.path())
        .env("HASHB_DEFAULT_STORAGE", workspace.path())
        .arg("--store")
        .arg(&store_path)
        .arg("put")
        .write_stdin(input.as_slice());

    cmd.assert()
        .success()
        .stdout(predicate::str::starts_with(expected_hash.as_str()));

    let (dir, file_name) = expected_hash.split_at(2);

    let stored_file_path = store_path.join("objects").join(dir).join(file_name);

    assert!(stored_file_path.is_file());

    Ok(())
}

#[test]
fn test_put_fails_if_custom_store_does_not_exist() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = tempdir()?;
    let store_path = workspace.path().join("new_store");

    assert!(!store_path.exists());

    let mut cmd = Command::cargo_bin(BIN_NAME)?;

    cmd.current_dir(workspace.path())
        .env("HASHB_DEFAULT_STORAGE", workspace.path())
        .arg("--store")
        .arg(&store_path)
        .arg("put")
        .write_stdin("test missing custom store");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("no such file or directory"));

    assert!(!store_path.exists());

    Ok(())
}
