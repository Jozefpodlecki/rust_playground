use anyhow::*;
use std::path::Path;


pub fn get_db_key(file_name: &str, xor_key: &[u8]) -> Result<[u8; 16]> {
    let file_stem = Path::new(file_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap();

    let prefix_end_index = file_stem.find('_').map(|i| i + 1).unwrap_or(0);
    let remaining = &file_stem[prefix_end_index..];

    let bytes = remaining.as_bytes();

    let key_bytes  = md5::compute(bytes);
    let mut key_bytes   = key_bytes .0;

    for i in 0..key_bytes.len() {
        let xor_index = 15 - i;
        key_bytes[i] ^= xor_key[xor_index];
    }

    key_bytes.reverse();

    Ok(key_bytes)
}