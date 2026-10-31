use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use crate::{
    errors::{Error, Result},
    hash::ObjectId,
    object::Object,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ObjectStore {
    root: PathBuf,
}

impl ObjectStore {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    pub fn put(&self, object: &Object) -> Result<()> {
        if !self.root.exists() {
            return Err(Error::InvalidPath { path: self.root });
        }

        let obj_path = self.object_path(object.id());

        if let Some(parent) = obj_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(obj_path);

        match file {
            Ok(mut file) => file.write_all(object.data())?,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                return Ok(());
            }
            Err(error) => return Err(error),
        }

        Ok(())
    }

    fn object_path(&self, id: ObjectId) -> PathBuf {
        let hash = id.to_string();

        self.root
            .join(".cas/objects")
            .join(&hash[..2])
            .join(&hash[2..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_path_uses_hash_prefix() -> io::Result<()> {
        let dir = tempfile::tempdir()?;
        let store = ObjectStore::new(dir.path());
        let id = ObjectId::new(b"Hello World");
        let hash = id.to_string();

        let expected = dir
            .path()
            .join(".cas/objects")
            .join(&hash[..2])
            .join(&hash[2..]);

        assert_eq!(store.object_path(id), expected);

        Ok(())
    }

    #[test]
    fn put_stores_object() -> io::Result<()> {
        let dir = tempfile::tempdir()?;
        let store = ObjectStore::new(dir.path());
        let object = Object::new(b"Hello World");

        store.put(&object)?;

        let path = store.object_path(object.id());

        assert!(path.is_file());
        assert_eq!(fs::read(path)?, object.data());

        Ok(())
    }

    #[test]
    fn put_creates_parent_directories() -> io::Result<()> {
        let dir = tempfile::tempdir()?;
        let store = ObjectStore::new(dir.path());
        let object = Object::new(b"Hello World");

        store.put(&object)?;

        let path = store.object_path(object.id());

        assert!(path.parent().unwrap().is_dir());

        Ok(())
    }

    #[test]
    fn put_existing_object_succeeds() -> io::Result<()> {
        let dir = tempfile::tempdir()?;
        let store = ObjectStore::new(dir.path());
        let object = Object::new(b"Hello World");

        store.put(&object)?;
        store.put(&object)?;

        assert_eq!(fs::read(store.object_path(object.id()))?, object.data());

        Ok(())
    }

    #[test]
    fn put_does_not_overwrite_existing_object() -> io::Result<()> {
        let dir = tempfile::tempdir()?;
        let store = ObjectStore::new(dir.path());
        let object = Object::new(b"Hello World");
        let path = store.object_path(object.id());

        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(&path, b"Existing contents")?;

        store.put(&object)?;

        assert_eq!(fs::read(path)?, b"Existing contents");

        Ok(())
    }

    #[test]
    fn put_empty_object() -> io::Result<()> {
        let dir = tempfile::tempdir()?;
        let store = ObjectStore::new(dir.path());
        let object = Object::new(b"");

        store.put(&object)?;

        let path = store.object_path(object.id());

        assert!(path.is_file());
        assert!(fs::read(path)?.is_empty());

        Ok(())
    }

    #[test]
    fn put_fails_when_root_does_not_exist() -> io::Result<()> {
        let dir = tempfile::tempdir()?;
        let store = ObjectStore::new(dir.path().join("missing"));
        let object = Object::new(b"Hello World");

        let error = store.put(&object).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::NotFound);

        Ok(())
    }
}
