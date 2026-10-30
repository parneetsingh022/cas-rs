pub mod hash;

use hex;
use sha2::{Digest, Sha256};

fn main() {
    let data = b"hello world";

    let result = Sha256::digest(data);
    let hash_string = hex::encode(result);
    println!("SHA-256: {}", hash_string);
}
