use std::{env, fs::File, io::{Cursor, Read}, sync::Arc};
use anyhow::*;
use aes::Aes256;
use blowfish::Blowfish;
use byteorder::{LittleEndian, ReadBytesExt};
use hex_literal::hex;

// use crate::lpk_entry::{GlobalConfiguration, LpkEntry};

mod lpk_entry;

fn main() -> Result<()> {
    dotenvy::dotenv()?;

    let cipher_key = option_env!("CIPHER_KEY").unwrap();;
    let aes_xor_key = option_env!("AES_XOR_KEY").unwrap();;
    let path = option_env!("LPK_PATH").unwrap();
    let mut reader = File::open(path)?;

    let file_count = reader.read_i32::<LittleEndian>()?;

    let entry_size = 528;
    let entries_size = file_count * entry_size;
    
    let mut entry_bytes = vec![0u8; entries_size as usize];
    reader.read_exact(&mut entry_bytes)?;


    // let blowfish = Blowfish::new_varkey(b"your_blowfish_key").unwrap();
    // let decrypted_bytes = blowfish.decrypt_ecb(&entry_bytes);

    // // 3) Create a stream + reader for decrypted bytes
    // let mut stream = Cursor::new(decrypted_bytes);

    // // 4) Setup AES cryptographic object (key/iv can be placeholder for now)
    // let key = [0u8; 32]; // 256 bits
    // let iv = [0u8; 16];  // 128 bits
    // let aes = Arc::new(Aes256::new_from_slice(&key).unwrap());
    // let crypto_object = Arc::new(Cbc::<Aes256, NoPadding>::new_from_slices(&key, &iv).unwrap());

    // // 5) Track offset
    // let offset = entries_size as u64 + 8;

    // let mut entry = LpkEntry::new(
    //         Arc::clone(&aes),
    //         Arc::new(blowfish.clone()),
    //         File::open("your_input_file.lpk")?,
    //         &mut stream,
    //         offset,
    //         i,
    //     )?;

    // let entry = LpkEntry::new(
    //     cryptographic_object, blowfish, lpk_reader, reader, offset, file_order)

    Ok(())
}
