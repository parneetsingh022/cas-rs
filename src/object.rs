use crate::errors::{Error, Result};

/// A SHA-256 identifier for content-addressed objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId([u8; 32]);

impl ObjectId {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl std::fmt::Display for ObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in &self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl std::str::FromStr for ObjectId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut bytes = [0u8; 32];
        hex::decode_to_slice(s, &mut bytes)?;
        Ok(Self(bytes))
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ObjectKind {
    Blob,
}

impl std::fmt::Display for ObjectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectKind::Blob => write!(f, "blob"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Object {
    id: ObjectId,
    kind: ObjectKind,
    size: usize,
}

impl Object {
    pub fn new(id: ObjectId, kind: ObjectKind, size: usize) -> Self {
        Self { id, kind, size }
    }

    pub fn id(&self) -> ObjectId {
        self.id
    }

    pub fn kind(&self) -> ObjectKind {
        self.kind
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

#[cfg(test)]
mod object_id_tests {
    use super::*;

    #[test]
    fn display_formats_as_hex_string() {
        let bytes = [0xab; 32];
        let id = ObjectId::new(bytes);

        let expected = "ab".repeat(32);
        assert_eq!(id.to_string(), expected);
    }

    #[test]
    fn object_id_roundtrip() {
        let bytes = [0x12; 32];
        let original = ObjectId::new(bytes);

        let parsed: ObjectId = original.to_string().parse().unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn invalid_object_id_is_rejected() {
        assert!("invalid".parse::<ObjectId>().is_err());
        assert!("abcd".parse::<ObjectId>().is_err());
        assert!("z".repeat(64).parse::<ObjectId>().is_err());
    }

    #[test]
    fn invalid_object_id_returns_hex_parse_error() {
        let err_length = "abcd".parse::<ObjectId>().unwrap_err();
        assert!(matches!(err_length, Error::HexParse(_)));

        let err_chars = "z".repeat(64).parse::<ObjectId>().unwrap_err();
        assert!(matches!(err_chars, Error::HexParse(_)));
    }
}

#[cfg(test)]
mod object_tests {
    use super::*;

    #[test]
    fn object_creation_and_getters() {
        // Using a dummy 32-byte array for the ObjectId dependency
        let id = ObjectId::new([0xab; 32]);
        let kind = ObjectKind::Blob;
        let size = 1024;

        let object = Object::new(id, kind, size);

        assert_eq!(object.id(), id);
        assert_eq!(object.kind(), ObjectKind::Blob);
        assert_eq!(object.size(), 1024);
    }

    #[test]
    fn object_equality() {
        let id1 = ObjectId::new([0x01; 32]);
        let id2 = ObjectId::new([0x02; 32]);

        let obj1 = Object::new(id1, ObjectKind::Blob, 512);
        let obj2 = Object::new(id1, ObjectKind::Blob, 512);
        let obj3 = Object::new(id2, ObjectKind::Blob, 512);
        let obj4 = Object::new(id1, ObjectKind::Blob, 1024); // Different size

        assert_eq!(obj1, obj2);
        assert_ne!(obj1, obj3); // Fails on ID
        assert_ne!(obj1, obj4); // Fails on size
    }

    #[test]
    fn object_clone() {
        let id = ObjectId::new([0xff; 32]);
        let original = Object::new(id, ObjectKind::Blob, 2048);

        #[allow(clippy::redundant_clone)]
        let cloned = original.clone();

        assert_eq!(original, cloned);
    }
}
