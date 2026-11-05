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

pub const STORAGE_BASE_DIR: &str = ".hsb";

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ObjectStore {
    root: PathBuf,
    temp_dir: PathBuf,
    objects_dir: PathBuf,
}

impl ObjectStore {
    pub fn new(root: impl AsRef<Path>) -> Result<Self> {
        let root_path = root.as_ref();

        if !root_path.exists() {
            return Err(Error::InvalidPath {
                path: root_path.to_path_buf(),
            });
        }

        let base_path = root_path.join(STORAGE_BASE_DIR);

        let temp_dir = base_path.join("tmp");
        let objects_dir = base_path.join("objects");

        // We only check/create temp once to avoid
        // IO checking overhead during put.
        fs::create_dir_all(&temp_dir)?;
        fs::create_dir_all(&objects_dir)?;

        Ok(Self {
            root: root_path.to_path_buf(),
            temp_dir,
            objects_dir,
        })
    }

    pub fn put(&self, mut source: impl Read) -> Result<Object> {
        let mut buffer = [0u8; 64 * 1024]; // 64 kb buffer

        self.put_with_buffer(&mut source, &mut buffer)
    }

    fn put_with_buffer(&self, mut source: impl Read, buffer: &mut [u8]) -> Result<Object> {
        let mut temp = NamedTempFile::new_in(&self.temp_dir)?;
        let mut hasher = Hasher::new();
        let mut size: usize = 0;

        // Stream chunk-by-chunk from the source to BOTH the temp file and hasher
        loop {
            let bytes_read = match source.read(buffer) {
                Ok(0) => break, // EOF
                Ok(n) => n,
                // Retry immediately if the read was interrupted by an OS signal
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e.into()),
            };

            let chunk = &buffer[..bytes_read];
            hasher.update(chunk);
            temp.write_all(chunk)?;

            size += bytes_read;
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

        self.objects_dir.join(&hash[..2]).join(&hash[2..])
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

    /// A mock reader that simulates an OS interrupt once before yielding data.
    struct InterruptingReader {
        has_interrupted: bool,
        inner_data: &'static [u8],
    }

    impl Read for InterruptingReader {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if !self.has_interrupted {
                self.has_interrupted = true;
                return Err(io::Error::new(
                    io::ErrorKind::Interrupted,
                    "simulated OS interrupt",
                ));
            }
            let mut inner_slice = self.inner_data;
            let bytes_read = inner_slice.read(buf)?;
            self.inner_data = inner_slice;
            Ok(bytes_read)
        }
    }

    /// A mock reader that simulates an unrecoverable I/O error on every read attempt.
    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "simulated hard failure",
            ))
        }
    }

    #[test]
    fn new_initializes_internal_directories() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let store = ObjectStore::new(dir.path()).unwrap();

        let expected_base = dir.path().join(STORAGE_BASE_DIR);
        let expected_temp = expected_base.join("tmp");
        let expected_objects = expected_base.join("objects");

        // Verify physical creation
        assert!(expected_temp.is_dir());

        // Verify internal state
        assert_eq!(store.temp_dir, expected_temp);
        assert_eq!(store.objects_dir, expected_objects);

        Ok(())
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
            .join(STORAGE_BASE_DIR)
            .join("objects")
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
        let tmp_dir = dir.path().join(STORAGE_BASE_DIR).join("tmp");
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
    fn put_existing_object_cleans_up_temp_file() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let store = ObjectStore::new(dir.path()).unwrap();
        let data = b"Hello world";

        // write normally
        let object = store.put(&data[..])?;

        // Try to write again. It will write to a temp file,
        // realize the hash already exists, and discard the temp file.
        store.put(&data[..])?;

        // Verify the objects directory has exactly 1 file (no duplicates)
        let obj_path = store.object_path(object.id());
        assert_eq!(fs::read_dir(obj_path.parent().unwrap())?.count(), 1);

        // Verify the tmp directory is completely empty
        let tmp_dir = dir.path().join(STORAGE_BASE_DIR).join("tmp");
        assert_eq!(fs::read_dir(tmp_dir)?.count(), 0);

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

    #[test]
    fn put_recovers_from_interrupted_read() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let store = ObjectStore::new(dir.path()).unwrap();

        let data = b"Resilient streaming";
        let reader = InterruptingReader {
            has_interrupted: false,
            inner_data: data,
        };

        let object = store.put(reader).expect(
            "store.put failed: it should have ignored the Interrupted error and completed the read",
        );

        assert_eq!(object.size(), data.len());
        let obj_path = store.object_path(object.id());
        assert_eq!(std::fs::read(obj_path)?, data);

        Ok(())
    }

    #[test]
    fn put_handles_large_streams_across_chunk_boundaries() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let store = ObjectStore::new(dir.path()).unwrap();

        // Create a payload larger than the 64KB buffer
        let size = 100 * 1024;
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();

        let object = store.put(&data[..])?;

        assert_eq!(object.size(), size);

        let path = store.object_path(object.id());
        assert_eq!(fs::read(path)?, data);

        Ok(())
    }

    #[test]
    fn put_bubbles_up_hard_read_errors() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let store = ObjectStore::new(dir.path()).unwrap();

        let result = store.put(FailingReader);

        assert!(result.is_err());
        if let Err(Error::Io(e)) = result {
            assert_eq!(e.kind(), io::ErrorKind::PermissionDenied);
        } else {
            panic!("Expected Io error variant");
        }

        Ok(())
    }
}
