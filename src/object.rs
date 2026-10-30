use std::{fs, io, path::Path};

use crate::hash::ObjectId;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Object {
    id: ObjectId,
    data: Vec<u8>,
}

impl Object {
    pub fn new(data: &[u8]) -> Self {
        Self {
            id: ObjectId::new(data),
            data: data.to_vec(),
        }
    }

    pub fn from_file(path: impl AsRef<Path>) -> io::Result<Self> {
        let data = fs::read(path)?;
        Ok(Self::new(&data))
    }

    pub fn data(&self) -> &[u8] {
        self.data.as_ref()
    }

    pub fn id(&self) -> ObjectId {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_from_bytes() {
        let data = b"Hello World";
        let object = Object::new(data);

        assert_eq!(object.data, data);
        assert_eq!(object.id, ObjectId::new(data));
    }

    #[test]
    fn empty_object() {
        let object = Object::new(b"");

        assert!(object.data.is_empty());
        assert_eq!(object.id, ObjectId::new(b""));
    }

    #[test]
    fn binary_object() {
        let data = [0x00, 0x01, 0xFF, 0x80];
        let object = Object::new(&data);

        assert_eq!(object.data, data);
        assert_eq!(object.id, ObjectId::new(data));
    }

    #[test]
    fn object_owns_data() {
        let mut data = vec![1, 2, 3];
        let object = Object::new(&data);

        data[0] = 99;

        assert_eq!(object.data, [1, 2, 3]);
    }

    #[test]
    fn object_from_file() -> io::Result<()> {
        let mut file = tempfile::NamedTempFile::new()?;
        use std::io::Write;

        file.write_all(b"Hello World")?;

        let object = Object::from_file(file.path())?;

        assert_eq!(object.data, b"Hello World");
        assert_eq!(object.id, ObjectId::new(b"Hello World"));

        Ok(())
    }

    #[test]
    fn object_from_empty_file() -> io::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        let object = Object::from_file(file.path())?;

        assert!(object.data.is_empty());
        assert_eq!(object.id, ObjectId::new(b""));

        Ok(())
    }

    #[test]
    fn object_from_missing_file() -> io::Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("missing.txt");

        let result = Object::from_file(path);

        assert!(matches!(
            result,
            Err(ref error) if error.kind() == io::ErrorKind::NotFound
        ));

        Ok(())
    }
}
