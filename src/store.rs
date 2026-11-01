use std::{
    fs,
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
            return Err(Error::InvalidPath {
                path: self.root.to_path_buf(),
            });
        }

        let obj_path = self.object_path(object.id());
        let parent = obj_path.parent().expect("object path has a parent");
        fs::create_dir_all(parent)?;

        if let Ok(existing) = fs::read(&obj_path) {
            if existing == object.data() {
                return Ok(());
            }
        }

        let mut temp_file = tempfile::NamedTempFile::new_in(parent)?;
        temp_file.write_all(object.data())?;
        temp_file.persist(&obj_path).map_err(|error| error.error)?;

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
    fn put_stores_object() -> Result<()> {
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
    fn put_creates_parent_directories() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let store = ObjectStore::new(dir.path());
        let object = Object::new(b"Hello World");

        store.put(&object)?;

        let path = store.object_path(object.id());

        assert!(path.parent().unwrap().is_dir());

        Ok(())
    }

    #[test]
    fn put_existing_object_succeeds() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let store = ObjectStore::new(dir.path());
        let object = Object::new(b"Hello World");

        store.put(&object)?;
        store.put(&object)?;

        assert_eq!(fs::read(store.object_path(object.id()))?, object.data());

        Ok(())
    }

    #[test]
    fn put_repairs_incomplete_existing_object() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let store = ObjectStore::new(dir.path());
        let object = Object::new(b"Hello World");
        let path = store.object_path(object.id());

        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(&path, b"Existing contents")?;

        store.put(&object)?;

        assert_eq!(fs::read(path)?, object.data());

        Ok(())
    }

    #[test]
    fn put_empty_object() -> Result<()> {
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
    fn put_fails_when_root_does_not_exist() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let store = ObjectStore::new(dir.path().join("missing"));
        let object = Object::new(b"Hello World");

        let error = store.put(&object).unwrap_err();

        assert!(matches!(error, Error::InvalidPath { .. }));
        Ok(())
    }
}
