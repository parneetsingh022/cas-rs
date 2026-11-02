use crate::object::ObjectId;

pub struct Hasher {
    inner: blake3::Hasher,
}

impl Hasher {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            inner: blake3::Hasher::new(),
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        self.inner.update(data);
    }

    pub fn finalize(&self) -> ObjectId {
        ObjectId::new(*self.inner.finalize().as_bytes())
    }
}

#[cfg(test)]
mod hasher_tests {
    use super::*;
    #[test]
    fn empty_hasher_produces_correct_hash() {
        let hasher = Hasher::new();
        let id = hasher.finalize();

        // Compare against the standard blake3 one-shot hash for an empty byte array
        let expected_bytes = *blake3::hash(b"").as_bytes();
        assert_eq!(id, ObjectId::new(expected_bytes));
    }

    #[test]
    fn single_update_produces_correct_hash() {
        let mut hasher = Hasher::new();
        hasher.update(b"Hello World");
        let id = hasher.finalize();

        let expected_bytes = *blake3::hash(b"Hello World").as_bytes();
        assert_eq!(id, ObjectId::new(expected_bytes));
    }

    #[test]
    fn incremental_updates_match_single_update() {
        let mut chunked_hasher = Hasher::new();
        chunked_hasher.update(b"Hello");
        chunked_hasher.update(b" ");
        chunked_hasher.update(b"World");
        let chunked_id = chunked_hasher.finalize();

        let mut single_hasher = Hasher::new();
        single_hasher.update(b"Hello World");
        let single_id = single_hasher.finalize();

        // Proves that streaming chunks produces the exact same ID as a bulk write
        assert_eq!(chunked_id, single_id);
    }

    #[test]
    fn finalize_does_not_consume_hasher() {
        let mut hasher = Hasher::new();
        hasher.update(b"Initial data");

        let id1 = hasher.finalize();

        // Finalize takes &self, so it shouldn't consume or mutate the state
        let id2 = hasher.finalize();
        assert_eq!(id1, id2);

        // We should be able to continue streaming after finalizing
        hasher.update(b" and more data");
        let id3 = hasher.finalize();

        let expected_bytes = *blake3::hash(b"Initial data and more data").as_bytes();
        assert_eq!(id3, ObjectId::new(expected_bytes));
        assert_ne!(id1, id3);
    }
}
