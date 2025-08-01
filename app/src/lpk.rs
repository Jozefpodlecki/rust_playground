use std::{borrow::Cow, collections::HashMap, fs::{self, File}, io::{Cursor, Read, Seek, SeekFrom}, path::{Path, PathBuf}};
use anyhow::*;
use ::blowfish::BlowfishLE;
use byteorder::{LittleEndian, ReadBytesExt};
use cipher::{block_padding::NoPadding, BlockDecryptMut, KeyInit, KeyIvInit};
use flate2::bufread::ZlibDecoder;
use sha2::{Digest, Sha256};

use crate::{lpk, types::RunArgs};

pub const CHUNK_SIZE: u32 = 1024;
pub const ENTRY_SIZE: u32 = 528;

#[derive(Debug)]
pub struct BlowfishZlib<'a>(Cow<'a, [u8]>, &'a [u8], usize);

#[derive(Debug)]
pub struct Aes256Cbc<'a>(Cow<'a, [u8]>, &'a [u8], &'a str);

#[derive(Debug)]
pub enum LpkEntryContent<'a> {
    BlowfishZlib(BlowfishZlib<'a>),
    Aes256Cbc(Aes256Cbc<'a>)
}

#[derive(Debug)]
pub struct LpkInfo<'a> {
    pub name: String,
    pub file_path: PathBuf,
    buffer: Option<Vec<u8>>,
    entries_metadata: Option<Vec<EntryMetadata>>,
    cipher_key: &'a [u8],
    aes_xor_key: &'a [u8]
}

#[derive(Debug)]
pub struct LpkEntry<'a> {
    pub content: LpkEntryContent<'a>,
    pub metadata: &'a EntryMetadata
}

#[derive(Debug, Clone)]
pub enum LpkEntryContentType {
    Unknown,
    Database
}

#[derive(Debug, Clone)]
pub struct EntryMetadata {
    pub order: u32,
    pub file_path: String,
    pub file_name: String,
    pub max_length: usize,
    pub length: usize,
    pub content_type: LpkEntryContentType,
    pub offset: usize,
    pub is_blowfish: bool,
}

pub fn get_lpks<'a>(args: &'a RunArgs) -> Result<Vec<LpkInfo<'a>>> {
    let RunArgs {
        lpk_dir,
        cipher_key,
        aes_xor_key,
        output_path,
        ..
    } = args;

    let output_path = Path::new(&output_path);
    let mut items = vec![];

    for entry in fs::read_dir(&lpk_dir)? {
        let entry = entry?;
        let file_path = entry.path();

        if file_path.extension().and_then(|s| s.to_str()) != Some("lpk") {
            continue;
        }

        let lpk_info = LpkInfo::new(file_path, cipher_key, aes_xor_key)?;
        items.push(lpk_info);
    }

    Ok(items)
}

pub fn get_lpks_dict<'a>(args: &'a RunArgs) -> Result<HashMap<String, LpkInfo<'a>>> {
    let lpks = get_lpks(args)?;
    let map = lpks
        .into_iter()
        .map(|lpk| (lpk.name.to_string(), lpk))
        .collect();
    Ok(map)
}

impl<'a> LpkInfo<'a> {
    fn read_file_to_buffer(file_path: &Path) -> Result<Vec<u8>> {
        let mut reader = File::open(file_path)?;
        let file_size = reader.metadata()?.len() as usize;
        let mut buffer = Vec::with_capacity(file_size);
        reader.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    fn decrypt_entries<'b>(cipher_key: &[u8], entry_bytes: &'b mut [u8]) -> Result<()> {
        let cipher = ecb::Decryptor::<BlowfishLE>::new_from_slice(cipher_key)
            .map_err(|e| anyhow::anyhow!("An error occurred whilst decrypting: {:?}", e))?;
        cipher.decrypt_padded_mut::<NoPadding>(entry_bytes)
            .map_err(|e| anyhow::anyhow!("An error occurred whilst decrypting: {:?}", e))?;
        Ok(())
    }

    fn parse_file_path(cursor: &mut Cursor<&Vec<u8>>) -> Result<(String, String, String)> {
        let file_path_length = cursor.read_i32::<LittleEndian>()?;
        let mut chars_buf = vec![0u8; file_path_length as usize];
        cursor.read_exact(&mut chars_buf)?;
        let file_path = str::from_utf8(&chars_buf)?
            .trim_start_matches(|c| c == '\\' || c == '.' || c == '/')
            .to_string();

        let path = Path::new(&file_path);
        let file_name = path.file_name()
            .and_then(|ext| ext.to_str())
            .unwrap()
            .to_string();
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap()
            .to_string();

        Ok((file_path, file_name, extension))
    }

    fn read_entry_metadata(cursor: &mut Cursor<&Vec<u8>>, order: u32, offset: usize) -> Result<EntryMetadata> {
        let (file_path, file_name, extension) = Self::parse_file_path(cursor)?;

        let position = order * ENTRY_SIZE - 12;
        cursor.seek(SeekFrom::Start(position as u64))?;

        let max_length = cursor.read_i32::<LittleEndian>()? as usize;
        let length = cursor.read_u32::<LittleEndian>()? as usize;
        let compressed_or_encrypted = cursor.read_i32::<LittleEndian>()? != 0;

        let content_type = match extension.as_str() {
            "db" => LpkEntryContentType::Database,
            _ => LpkEntryContentType::Unknown
        };

        Ok(EntryMetadata {
            order,
            file_path,
            file_name,
            max_length,
            length,
            content_type,
            offset,
            is_blowfish: compressed_or_encrypted,
        })
    }

    fn parse_entries_metadata(entry_bytes: &Vec<u8>, file_count: u32, entries_size: usize) -> Result<Vec<EntryMetadata>> {
        let mut cursor = Cursor::new(entry_bytes);
        let mut entries_metadata = Vec::new();
        let mut offset = (entries_size + 8) as usize;

        for order in 1..=file_count {
          
            let mut entry = Self::read_entry_metadata(&mut cursor, order, offset)?;
            offset += entry.length;
            entries_metadata.push(entry);
        }

        Ok(entries_metadata)
    }

    pub fn new(file_path: PathBuf, cipher_key: &'a [u8], aes_xor_key: &'a [u8]) -> Result<Self> {
        let name = file_path.file_stem().unwrap().to_string_lossy().to_string();

        Ok(Self {
            name,
            file_path,
            buffer: None,
            entries_metadata: None,
            aes_xor_key,
            cipher_key,
        })
    }

    // pub fn new(file_path: &str, cipher_key: &'a [u8], aes_xor_key: &'a [u8]) -> Result<Self> {

    //     let mut reader = File::open(file_path)?;
    //     let file_size = reader.metadata()?.len() as usize;
    //     let mut buffer = Vec::with_capacity(file_size);
    //     reader.read_to_end(&mut buffer)?;

    //     let mut reader = Cursor::new(&buffer);
    //     let file_count = reader.read_u32::<LittleEndian>()?;
    //     let entries_size = file_count * ENTRY_SIZE;
    //     let mut offset = (entries_size + 8) as usize;

    //     let mut entry_bytes: Vec<u8> = vec![0u8; entries_size as usize];
    //     reader.read_exact(&mut entry_bytes)?;

    //     let cipher = ecb::Decryptor::<BlowfishLE>::new_from_slice(&cipher_key)
    //         .map_err(|e| anyhow::anyhow!("An error occurred whilst decrypting: {:?}", e))?;
    //     let decrypted = cipher.decrypt_padded_mut::<NoPadding>(&mut entry_bytes)
    //         .map_err(|e| anyhow::anyhow!("An error occurred whilst decrypting: {:?}", e))?;

    //     let mut cursor = Cursor::new(decrypted);

    //     let mut entries_metadata = vec![];

    //     for order in 1..=file_count {

    //         let relative_file_path_length = cursor.read_i32::<LittleEndian>()?;
    //         let mut chars_buf = vec![0u8; relative_file_path_length as usize];
    //         cursor.read_exact(&mut chars_buf)?;
            
    //         let file_path = String::from_utf8(chars_buf)?;

    //         let (file_name, extension) = {
    //             let path = Path::new(&file_path);

    //             let file_name = path.file_name()
    //                 .and_then(|ext| ext.to_str())
    //                 .unwrap()
    //                 .to_string();

    //             let extension = path.extension()
    //                 .and_then(|ext| ext.to_str())
    //                 .unwrap()
    //                 .to_string();

    //             (file_name, extension)
    //         };

    //         let position = order * ENTRY_SIZE - 12;
    //         cursor.seek(SeekFrom::Start(position as u64))?;

    //         let max_length = cursor.read_i32::<LittleEndian>()? as usize;
    //         let length = cursor.read_u32::<LittleEndian>()? as usize;
    //         let content_type = extension.to_string();
    //         let compressed_or_encrypted = cursor.read_i32::<LittleEndian>()? != 0;

    //         let entry = EntryMetadata {
    //             order,
    //             file_path,
    //             file_name,
    //             max_length,
    //             length,
    //             content_type,
    //             offset,
    //             is_blowfish: compressed_or_encrypted
    //         };

    //         entries_metadata.push(entry);

    //         offset += length;
    //     }

    //     Ok(Self {
    //         buffer,
    //         entries_metadata,
    //         aes_xor_key,
    //         cipher_key
    //     })
    // }

    pub fn load(&mut self) -> Result<()> {
        let buffer = Self::read_file_to_buffer(&self.file_path)?;
        let mut reader = Cursor::new(&buffer);

        let file_count = reader.read_u32::<LittleEndian>()?;
        let entries_size = file_count * ENTRY_SIZE;

        let mut entry_bytes = vec![0u8; entries_size as usize];
        reader.read_exact(&mut entry_bytes)?;
        Self::decrypt_entries(self.cipher_key, &mut entry_bytes)?;

        self.buffer = Some(buffer);
        let entries_metadata = Self::parse_entries_metadata(
            &entry_bytes,
            file_count,
            entries_size as usize)?;
        self.entries_metadata = Some(entries_metadata);

        Ok(())
    }
        
    pub fn get_summary(&self) -> &Vec<EntryMetadata> {
        let entries_metadata = self.entries_metadata.as_ref().expect("Call \"load()\" first");
        entries_metadata
    }

    pub fn get_entries(&self) -> Vec<LpkEntry> {
        let buffer = self.buffer.as_ref().expect("Call \"load()\" first");

        self.entries_metadata.iter().flatten().map(|meta| {
            let content_slice = &buffer[meta.offset..meta.offset + meta.length];

            let content = if meta.is_blowfish {
                LpkEntryContent::BlowfishZlib(BlowfishZlib(Cow::Borrowed(content_slice), &self.cipher_key, meta.max_length))
            } else {
                LpkEntryContent::Aes256Cbc(Aes256Cbc(Cow::Borrowed(content_slice), self.aes_xor_key, &meta.file_name))
            };

            LpkEntry {
                content,
                metadata: meta
            }
        }).collect()
    }
}

impl<'a> LpkEntryContent<'a> {
    pub fn to_bytes(&'a mut self) -> Result<Cow<'a, [u8]>> {
        match self {
            crate::lpk::LpkEntryContent::BlowfishZlib(content) => content.to_bytes(),
            crate::lpk::LpkEntryContent::Aes256Cbc(content) => content.to_bytes(),
        }
    }
}

impl<'a> BlowfishZlib<'a> {
    pub fn to_bytes(&'a mut self) -> Result<Cow<'a, [u8]>> {
        let encrypted = self.0.to_mut();
        let cipher = ecb::Decryptor::<BlowfishLE>::new_from_slice(self.1)
            .map_err(|e| anyhow::anyhow!("An error occurred whilst decrypting: {:?}", e))?;
        let decrypted = cipher.decrypt_padded_mut::<NoPadding>(encrypted)
            .map_err(|e| anyhow::anyhow!("An error occurred whilst decrypting: {:?}", e))?;

        let mut decoder = ZlibDecoder::new(decrypted);
        let mut decompressed = Vec::with_capacity(self.2 as usize);
        decoder.read_to_end(&mut decompressed)?;
        
        Ok(Cow::Owned(decompressed))
    }
}

impl<'a> Aes256Cbc<'a> {
    fn get_db_key(file_name: &str, xor_key: &[u8]) -> Result<Vec<u8>> {
        let file_stem = Path::new(file_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap();

        let prefix_end_index = file_stem.find('_').map(|i| i + 1).unwrap_or(0);
        let remaining = &file_stem[prefix_end_index..];

        let bytes: Vec<u8> = remaining.encode_utf16()
            .flat_map(|c| c.to_le_bytes())
            .collect();

        let key_bytes  = md5::compute(bytes);
        let mut key_bytes = key_bytes .0;

        for i in 0..key_bytes.len() {
            let xor_index = 15 - i;
            key_bytes[i] ^= xor_key[xor_index];
        }

        key_bytes.reverse();
       
        let hex_string = hex::encode(key_bytes);
        let ascii_bytes = hex_string.as_bytes();

        let key = Sha256::digest(ascii_bytes);
        let key = key.to_vec();

        Ok(key)
    }

    pub fn to_bytes(&'a mut self) -> Result<Cow<'a, [u8]>> {
        let encrypted = self.0.to_mut();
        let key = Self::get_db_key(self.2, self.1)?;
   
        let iv: [u8; 16] = [0; 16];

        let mut offset = 0;
        let decryptor = cbc::Decryptor::<aes::Aes256>::new_from_slices(&key, &iv)
            .map_err(|e| anyhow::anyhow!("An error occurred whilst decrypting: {:?}", e))?;

        let chunk_size = CHUNK_SIZE as usize;
        while offset + chunk_size <= encrypted.len() {
            
            let decryptor = decryptor.clone();
            let block = &mut encrypted[offset..offset + chunk_size];
            decryptor.decrypt_padded_inout_mut::<NoPadding>(block.into())
                .map_err(|e| anyhow::anyhow!("An error occurred whilst decrypting: {:?}", e))?;
            offset += chunk_size;
        }

        Ok(Cow::Borrowed(encrypted))
    }
}
