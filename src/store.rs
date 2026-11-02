use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use tempfile::NamedTempFile;

use crate::{
    errors::{Error, Result},
    hash::Hasher,
    object::{Object, ObjectId, ObjectKind},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ObjectStore {
    root: PathBuf,
}

impl ObjectStore {
    pub fn new(root: impl AsRef<Path>) -> Result<Self> {
        let root_path = root.as_ref();

        if !root_path.exists() {
            return Err(Error::InvalidPath {
                path: root_path.to_path_buf(),
            });
        }

        Ok(Self {
            root: root_path.to_path_buf(),
        })
    }

    pub fn put(&self, mut source: impl Read) -> Result<Object> {
        let temp_dir = self.root.join("tmp");
        fs::create_dir_all(&temp_dir)?;

        let mut temp = NamedTempFile::new_in(&temp_dir)?;
        let mut hasher = Hasher::new();
        let mut buffer = [0; 65536]; // 64KB buffer for streaming
        let mut size: usize = 0;

        // Stream chunk-by-chunk from the source to BOTH the temp file and hasher
        loop {
            let bytes_read = source.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let chunk = &buffer[..bytes_read];
            hasher.update(chunk);
            temp.write_all(chunk)?;

            size += bytes_read as usize;
        }

        let object_id = hasher.finalize();
        let object = Object::new(object_id, ObjectKind::Blob, size);
        let obj_path = self.object_path(object.id());

        // Early return pattern for identical objects
        if !obj_path.exists() {
            if let Some(parent) = obj_path.parent() {
                fs::create_dir_all(parent)?;
            }

            match temp.persist_noclobber(&obj_path) {
                Ok(_) => {}
                Err(error) if error.error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error.error.into()),
            }
        }

        Ok(object)
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

    /// Helper function purely for generating IDs in tests
    fn hash_test_data(data: &[u8]) -> ObjectId {
        let mut hasher = Hasher::new();
        hasher.update(data);
        hasher.finalize()
    }

    #[test]
    fn new_fails_if_root_does_not_exist() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let non_existent_path = dir.path().join("does_not_exist");
        let result = ObjectStore::new(&non_existent_path);

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn new_succeeds_if_root_exists() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let result = ObjectStore::new(dir.path());

        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn object_path_uses_hash_prefix() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let store = ObjectStore::new(dir.path()).unwrap();

        // Use the test helper here
        let id = hash_test_data(b"Hello World");
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
        let store = ObjectStore::new(dir.path()).unwrap();
        let data = b"Hello World";

        // put() now reads the stream and returns the Object metadata
        let object = store.put(&data[..])?;
        let path = store.object_path(object.id()); // Or object.id() if you made it private with a getter

        assert!(path.is_file());
        assert_eq!(fs::read(path)?, data);

        Ok(())
    }

    #[test]
    fn put_creates_parent_directories() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let store = ObjectStore::new(dir.path()).unwrap();

        let object = store.put(&b"Hello World"[..])?;
        let path = store.object_path(object.id());

        assert!(path.parent().unwrap().is_dir());

        Ok(())
    }

    #[test]
    fn put_existing_object_succeeds() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let store = ObjectStore::new(dir.path()).unwrap();
        let data = b"Hello World";

        let object1 = store.put(&data[..])?;
        let object2 = store.put(&data[..])?; // Second put

        assert_eq!(object1.id(), object2.id());
        assert_eq!(fs::read(store.object_path(object1.id()))?, data);

        Ok(())
    }

    #[test]
    fn put_does_not_overwrite_existing_object() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let store = ObjectStore::new(dir.path()).unwrap();
        let data = b"Hello World";

        // Predict the ID before we stream it
        let expected_id = hash_test_data(data);
        let path = store.object_path(expected_id);

        // Intentionally poison the cache
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(&path, b"Existing contents")?;

        // The engine should see the file exists and skip the rename
        store.put(&data[..])?;

        assert_eq!(fs::read(path)?, b"Existing contents");

        Ok(())
    }

    #[test]
    fn put_leaves_no_temp_files() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let store = ObjectStore::new(dir.path()).unwrap();
        let data = b"Hello World";

        let object = store.put(&data[..])?;
        store.put(&data[..])?;

        let path = store.object_path(object.id());

        // Ensure the objects directory only has the final blob
        assert_eq!(fs::read_dir(path.parent().unwrap())?.count(), 1);

        // Ensure our new dedicated `tmp/` folder is empty after streams finish
        let tmp_dir = dir.path().join("tmp");
        assert_eq!(fs::read_dir(tmp_dir)?.count(), 0);

        Ok(())
    }

    #[test]
    fn put_empty_object() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let store = ObjectStore::new(dir.path()).unwrap();

        let object = store.put(&b""[..])?;
        let path = store.object_path(object.id());

        assert!(path.is_file());
        assert!(fs::read(path)?.is_empty());
        assert_eq!(object.size(), 0);

        Ok(())
    }

    #[test]
    fn new_fails_when_root_does_not_exist() -> Result<()> {
        let dir = tempfile::tempdir()?;
        // Constructor should fail immediately instead of waiting for `put`
        let error = ObjectStore::new(dir.path().join("missing")).unwrap_err();

        assert!(matches!(error, Error::InvalidPath { .. }));
        Ok(())
    }

    #[test]
    fn test_put_existing_object_skips_write() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let store = ObjectStore::new(dir.path()).unwrap();
        let data = b"Hello world";

        let object = store.put(&data[..])?;

        let obj_path = store.object_path(object.id());
        let target_dir = obj_path.parent().unwrap_or(dir.path());

        let original_perm = fs::metadata(target_dir)?.permissions();
        let mut readonly_perm = original_perm.clone();

        // Set the permission of target_dir folder to readonly.
        // Now when we try to save an existing object, if we are
        // still writing a temp-object-file here, this test should fail.
        readonly_perm.set_readonly(true);
        fs::set_permissions(target_dir, readonly_perm)?;

        store.put(&data[..]).expect(
            "put() attempted to write a temporary file into a read-only object directory.\n",
        );

        // Restore permissions so the tempdir can be safely deleted on cleanup
        fs::set_permissions(target_dir, original_perm)?;

        Ok(())
    }

    #[test]
    fn put_tracks_object_size() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let store = ObjectStore::new(dir.path()).unwrap();

        // "Hello World" is exactly 11 bytes
        let data = b"Hello World";
        let object = store.put(&data[..])?;

        assert_eq!(object.size(), 11);

        Ok(())
    }
}
