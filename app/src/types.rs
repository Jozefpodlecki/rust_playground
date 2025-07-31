use std::{borrow::Cow, fs::File, io::{Cursor, Read, Seek, SeekFrom}, marker::PhantomData, path::Path};
use anyhow::*;
use ::blowfish::BlowfishLE;
use byteorder::{LittleEndian, ReadBytesExt};
use cipher::{block_padding::NoPadding, BlockDecryptMut, KeyInit, KeyIvInit};
use flate2::bufread::ZlibDecoder;
use sha2::{Digest, Sha256};

use crate::utils::get_db_key;

pub struct RunArgs {
    pub cipher_key: Vec<u8>,
    pub aes_xor_key: Vec<u8>,
    pub lpk_dir: String,
    pub output_path: String,
    pub exe_path: String,
    pub exe_args: Vec<String>,
}

#[derive(Debug)]
pub struct LpkInfo {
    pub entries: Vec<LpkEntryType>
}

impl LpkInfo {
    pub fn new(file_path: &str, cipher_key: &[u8]) -> Result<Self> {

        let mut reader = File::open(file_path)?;

        let file_count = reader.read_u32::<LittleEndian>()?;
        let entry_size = 528;
        let entries_size = file_count * entry_size;
        let mut offset = entries_size + 8;

        let mut entry_bytes: Vec<u8> = vec![0u8; entries_size as usize];
        reader.read_exact(&mut entry_bytes)?;

        let cipher = ecb::Decryptor::<BlowfishLE>::new_from_slice(&cipher_key)
            .map_err(|e| anyhow::anyhow!("An error occurred whilst decrypting: {:?}", e))?;
        let decrypted = cipher.decrypt_padded_mut::<NoPadding>(&mut entry_bytes)
            .map_err(|e| anyhow::anyhow!("An error occurred whilst decrypting: {:?}", e))?;

        let mut cursor = Cursor::new(decrypted);

        let mut entries = vec![];

        for order in 1..=file_count {

            let relative_file_path_length = cursor.read_i32::<LittleEndian>()?;
            let mut chars_buf = vec![0u8; relative_file_path_length as usize];
            cursor.read_exact(&mut chars_buf)?;
            
            let file_path = String::from_utf8(chars_buf)?;

            let (file_name, extension) = {
                let path = Path::new(&file_path);
                
                let file_name = path.file_name()
                    .and_then(|ext| ext.to_str())
                    .unwrap()
                    .to_string();

                let extension = path.extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap()
                    .to_string();

                (file_name, extension)
            };

            let position = order * entry_size - 12;
            cursor.seek(SeekFrom::Start(position as u64))?;

            let max_length = cursor.read_i32::<LittleEndian>()?;
            let length = cursor.read_u32::<LittleEndian>()?;
            let content_type = extension.to_string();
            let compressed_or_encrypted = cursor.read_i32::<LittleEndian>()? != 0;

            let mut content: Vec<u8> = vec![0u8; length as usize];
            reader.seek(SeekFrom::Start(offset as u64))?;
            reader.read_exact(&mut content)?;

            let entry = match compressed_or_encrypted {
                true => LpkEntryType::BlowfishCompressed(LpkEntry {
                    content,
                    order,
                    file_path,
                    file_name,
                    max_length,
                    length,
                    content_type,
                    offset,
                    _marker: PhantomData
                }),
                false => LpkEntryType::Aes256CbcEncrypted(LpkEntry {
                    content,
                    order,
                    file_path,
                    file_name,
                    max_length,
                    length,
                    content_type,
                    offset,
                    _marker: PhantomData
                }),
            };
            
            entries.push(entry);

            offset += length;
        }

        Ok(Self {
            entries
        })
    }
}

#[derive(Debug)]
pub struct BlowfishCompressed;

#[derive(Debug)]
pub struct Aes256CbcEncrypted;

#[derive(Debug)]
pub enum LpkEntryType {
    BlowfishCompressed(LpkEntry<BlowfishCompressed>),
    Aes256CbcEncrypted(LpkEntry<Aes256CbcEncrypted>)
}

#[derive(Debug)]
pub struct LpkEntry<D> {
    pub content: Vec<u8>,
    pub order: u32,
    pub file_path: String,
    pub file_name: String,
    pub max_length: i32,
    pub length: u32,
    pub content_type: String,
    pub offset: u32,
    pub _marker: PhantomData<D>
}

impl LpkEntry<BlowfishCompressed> {
    pub fn get_content<'a>(&'a mut self, cipher_key: &[u8]) -> Result<Cow<'a, [u8]>> {
        let cipher = ecb::Decryptor::<BlowfishLE>::new_from_slice(cipher_key)
            .map_err(|e| anyhow::anyhow!("An error occurred whilst decrypting: {:?}", e))?;
        let decrypted = cipher.decrypt_padded_mut::<NoPadding>(&mut self.content)
            .map_err(|e| anyhow::anyhow!("An error occurred whilst decrypting: {:?}", e))?;

        let mut decoder = ZlibDecoder::new(decrypted);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        
        Ok(Cow::Owned(decompressed))
    }
}

impl LpkEntry<Aes256CbcEncrypted> {
    pub fn get_content<'a>(&'a mut self, aes_xor_key: &[u8]) -> Result<Cow<'a, [u8]>> {
        let database_key = get_db_key(&self.file_name, aes_xor_key)?;
        let hex_string = hex::encode(database_key);
        let ascii_bytes = hex_string.as_bytes();
        let key = Sha256::digest(ascii_bytes);
        let iv: [u8; 16] = [0; 16];

        let mut offset = 0;
        let chunk_size = 1024;
        let decryptor = cbc::Decryptor::<aes::Aes256>::new_from_slices(&key, &iv)
            .map_err(|e| anyhow::anyhow!("An error occurred whilst decrypting: {:?}", e))?;

        while offset + chunk_size <= self.content.len() {
            
            let decryptor = decryptor.clone();
            let block = &mut self.content[offset..offset + chunk_size];
            decryptor.decrypt_padded_mut::<NoPadding>(block)
                .map_err(|e| anyhow::anyhow!("An error occurred whilst decrypting: {:?}", e))?;
            offset += chunk_size;
        }

        Ok(Cow::Borrowed(&self.content))
    }
}
