use std::{env, fs::File, io::{Cursor, Read, Seek, SeekFrom}, path::Path, sync::Arc};
use anyhow::*;
use aes::Aes256;
use ::blowfish::BlowfishLE;
use byteorder::{LittleEndian, ReadBytesExt};
use cipher::{block_padding::NoPadding, BlockDecryptMut, KeyInit};
use hex_literal::hex;

mod blowfishv2;
mod blowfish;
mod lpk_entry;

#[derive(Debug, Clone)]
pub struct LpkEntry {
    max_length: i32,
    length: i32,
    content_type: String,
    compressed_or_encrypted: bool,
}

impl LpkEntry {

}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;
    
    let cipher_key = std::env::var("CIPHER_KEY").unwrap();
    // let cipher_key = hex::decode(cipher_key).unwrap();
    let cipher_key = cipher_key.as_bytes();
    let aes_xor_key = std::env::var("AES_XOR_KEY").unwrap();
    let path = std::env::var("LPK_PATH").unwrap();

    let mut reader = File::open(path)?;
    let file_count = reader.read_i32::<LittleEndian>()?;
    let entry_size = 528;
    let entries_size = file_count * entry_size;

    println!("{file_count} {entries_size:?}");

    let mut entry_bytes: Vec<u8> = vec![0u8; entries_size as usize];
    reader.read_exact(&mut entry_bytes)?;

    println!("entry_bytes: {}", entry_bytes.len());
    // // 16636752

    let cipher = ecb::Decryptor::<BlowfishLE>::new_from_slice(&cipher_key).unwrap();
    let decrypted = cipher.decrypt_padded_mut::<NoPadding>(&mut entry_bytes).unwrap();

    let mut cursor = Cursor::new(decrypted);
    let relative_file_path_length = cursor.read_i32::<LittleEndian>()?;
    println!("{}", relative_file_path_length);
    
    let mut chars_buf = vec![0u8; relative_file_path_length as usize];
    cursor.read_exact(&mut chars_buf)?;
    
    let file_path = String::from_utf8(chars_buf).unwrap();

    cursor.seek(SeekFrom::Start(1 * entry_size as u64)).unwrap();
    cursor.seek(SeekFrom::Current(-12)).unwrap();

    let extension = Path::new(&file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap();

    let max_length = cursor.read_i32::<LittleEndian>()?;
    let length = cursor.read_i32::<LittleEndian>()?;
    let content_type = extension.to_string();
    let compressed_or_encrypted = cursor.read_i32::<LittleEndian>()? > 1;

    let entry = LpkEntry {
        max_length,
        length,
        content_type,
        compressed_or_encrypted
    };

    println!("{entry:?}");

    Ok(())
}
