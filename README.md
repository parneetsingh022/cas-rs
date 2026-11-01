# cas-rs
A lightweight content-addressable storage engine written in Rust.

## Usage

The following command stores a file into the local CAS:

```bash
cargo run -- <file_path>
```

The file is copied into the ./cas/objects/ directory relative to the current working directory. To prevent OS-level performance degradation from storing thousands of files in a single folder, cas-rs uses a two-level sharded directory structure.

The first two characters of the object's cryptographic hash create a parent subdirectory, and the remaining 62 characters form the file itself. This combined hash serves as the unique object ID for future retrieval:
```
./cas/objects/
├── a5/
│   └── 91a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e
├── e3/
│   └── b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
```
**Note:** Files are not compressed and stored as it is.
