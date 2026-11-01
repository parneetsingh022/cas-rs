use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use tempfile::NamedTempFile;

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

        // Early return if an object already exist.
        if obj_path.exists() {
            return Ok(());
        }

        if let Some(parent) = obj_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write to a temp file in the same directory so the final rename is atomic
        // and an interrupted write never leaves a partial object at its hash path.
        let mut temp = NamedTempFile::new_in(obj_path.parent().unwrap_or(&self.root))?;
        temp.write_all(object.data())?;

        match temp.persist_noclobber(&obj_path) {
            Ok(_) => Ok(()),
            Err(error) if error.error.kind() == io::ErrorKind::AlreadyExists => Ok(()),
            Err(error) => Err(error.error.into()),
        }
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
    fn put_does_not_overwrite_existing_object() -> Result<()> {
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
    fn put_leaves_no_temp_files() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let store = ObjectStore::new(dir.path());
        let object = Object::new(b"Hello World");

        store.put(&object)?;
        store.put(&object)?;

        let path = store.object_path(object.id());

        assert_eq!(fs::read_dir(path.parent().unwrap())?.count(), 1);

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

    #[test]
    fn test_put_existing_object_skips_write() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let store = ObjectStore::new(dir.path());
        let object = Object::new(b"Hello world");

        store.put(&object)?;

        let obj_path = store.object_path(object.id());
        let target_dir = obj_path.parent().unwrap_or(dir.path());

        // We sill set the permission of temp folder to readonly.
        // Now when we try to save an existing object, if we are
        // still writing a temp-object-file this test should fail.
        let original_perm = fs::metadata(target_dir)?.permissions();
        let mut readonly_perm = original_perm.clone();

        readonly_perm.set_readonly(true);
        fs::set_permissions(target_dir, readonly_perm)?;

        store
            .put(&object)
            .expect("put() attempted to write a temporary file for an already existing object.\n");

        // Restore permissions so the tempdir can be safely deleted on cleanup
        fs::set_permissions(target_dir, original_perm)?;

        Ok(())
    }
}
