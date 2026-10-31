//! Content-addressed object identifiers.
//!
//! This module provides [`ObjectId`], a 256-bit identifier derived from
//! the SHA-256 hash of an object's contents.
//!
//! Object identifiers can be converted to and from hexadecimal strings.

use crate::errors::{Error, Result};
use sha2::{Digest, Sha256};

/// A SHA-256 identifier for content-addressed objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId([u8; 32]);

impl ObjectId {
    pub fn new(data: impl AsRef<[u8]>) -> Self {
        Self(Sha256::digest(data.as_ref()).into())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_id_init() -> Result<()> {
        let obj_id = ObjectId::new("Hello World");
        let expected = "a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e";

        assert_eq!(obj_id.to_string(), expected);

        Ok(())
    }

    #[test]
    fn empty_content_has_correct_hash() {
        let id = ObjectId::new("");

        assert_eq!(
            id.to_string(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn string_and_bytes_produce_same_id() {
        let a = ObjectId::new("Hello World");
        let b = ObjectId::new(b"Hello World");

        assert_eq!(a, b);
    }

    #[test]
    fn different_content_produces_different_ids() {
        let a = ObjectId::new("hello");
        let b = ObjectId::new("Hello");

        assert_ne!(a, b);
    }

    #[test]
    fn object_id_roundtrip() {
        let original = ObjectId::new("Hello World");

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
