// use std::fs::File;
// use std::io::{self, Read, Seek, SeekFrom};
// use std::path::{Path, PathBuf};
// use std::fmt;
// use std::sync::Arc;

// use aes::Aes256;
// use blowfish::Blowfish;


// use flate2::read::ZlibDecoder;
// use md5;

// use tokio::io::AsyncReadExt;

// pub enum LpkEntryStorageType {
//     Encrypted,
//     Compressed,
// }

// pub enum LpkEntryContentType {
//     Bin,
//     Xml,
//     Db,
//     Enc,
//     Epf,
//     Loa,
//     Ttf,
// }

// pub struct LpkEntry {
//     cryptographic_object: Arc<Aes256>,
//     blowfish: Arc<Blowfish>,
//     lpk_reader: File,

//     pub order: u32,
//     pub file_path: PathBuf,
//     pub file_name: String,
//     pub max_length: u32,
//     pub length: u32,
//     pub offset: u64,
//     pub content_type: LpkEntryContentType,
//     pub storage_type: LpkEntryStorageType,
// }

// impl LpkEntry {
//     pub fn new(
//         cryptographic_object: Arc<Aes256>,
//         blowfish: Arc<Blowfish>,
//         mut lpk_reader: File,
//         mut reader: File,
//         offset: u64,
//         file_order: u32,
//     ) -> io::Result<Self> {
//         use std::io::BufReader;

//         let mut buf_reader = BufReader::new(&mut reader);

//         let mut int_buf = [0u8; 4];
//         buf_reader.read_exact(&mut int_buf)?;
//         let relative_file_path_length = u32::from_le_bytes(int_buf);

//         let mut chars = vec![0u8; relative_file_path_length as usize];
//         buf_reader.read_exact(&mut chars)?;
//         let relative_file_path = String::from_utf8_lossy(&chars).to_string();

//         let file_path = PathBuf::from(relative_file_path.replace("\\..\\", "").trim_start_matches('\\'));
//         let file_name = file_path.file_name().unwrap().to_string_lossy().to_string();

//         buf_reader.seek(SeekFrom::Start(file_order as u64 * GlobalConfiguration::ENTRY_SIZE as u64))?;
//         buf_reader.seek(SeekFrom::Current(-12))?;

//         buf_reader.read_exact(&mut int_buf)?;
//         let max_length = u32::from_le_bytes(int_buf);
//         buf_reader.read_exact(&mut int_buf)?;
//         let length = u32::from_le_bytes(int_buf);

//         let mut storage_buf = [0u8; 4];
//         buf_reader.read_exact(&mut storage_buf)?;
//         let storage_type = match u32::from_le_bytes(storage_buf) {
//             0 => LpkEntryStorageType::Encrypted,
//             _ => LpkEntryStorageType::Compressed,
//         };

//         let content_type = Self::get_content_type(&file_name)?;

//         Ok(Self {
//             cryptographic_object,
//             blowfish,
//             lpk_reader,
//             order: file_order,
//             file_path,
//             file_name,
//             max_length,
//             length,
//             offset,
//             content_type,
//             storage_type,
//         })
//     }

//     fn get_content_type(file_name: &str) -> Result<LpkEntryContentType, String> {
//         match Path::new(file_name)
//             .extension()
//             .and_then(|s| s.to_str())
//             .unwrap_or("")
//             .to_lowercase()
//             .as_str()
//         {
//             "bin" => Ok(LpkEntryContentType::Bin),
//             "xml" => Ok(LpkEntryContentType::Xml),
//             "db" => Ok(LpkEntryContentType::Db),
//             "enc" => Ok(LpkEntryContentType::Enc),
//             "epf" => Ok(LpkEntryContentType::Epf),
//             "loa" => Ok(LpkEntryContentType::Loa),
//             "ttf" => Ok(LpkEntryContentType::Ttf),
//             ext => Err(format!("Unsupported extension: {}", ext)),
//         }
//     }

//     fn get_db_key(file_name: &str, xor_key: &[u8]) -> Vec<u8> {
//         let file_stem = Path::new(file_name).file_stem().unwrap().to_string_lossy();
//         let prefix_end_index = file_stem.find('_').unwrap() + 1;
//         let relevant = &file_stem[prefix_end_index..];
//         let bytes = relevant.encode_utf16().flat_map(|u| u.to_le_bytes()).collect::<Vec<u8>>();

//         let mut hashed = md5::compute(&bytes).0;

//         for i in 0..hashed.len() {
//             let xor_index = 15 - i;
//             hashed[i] ^= xor_key[xor_index];
//         }

//         hashed.reverse();
//         hashed.to_vec()
//     }

//     pub async fn get_content(&mut self, aes_xor_key: &[u8]) -> io::Result<Vec<u8>> {
//         use std::io::Cursor;

//         self.lpk_reader.seek(SeekFrom::Start(self.offset))?;
//         let mut bytes = vec![0u8; self.length as usize];
//         self.lpk_reader.read_exact(&mut bytes)?;

//         match self.storage_type {
//             LpkEntryStorageType::Compressed => {
//                 // Example: decrypt with Blowfish and decompress with zlib
//                 let decrypted = self.blowfish.decrypt_ecb(&bytes);
//                 let mut decoder = ZlibDecoder::new(Cursor::new(decrypted));
//                 let mut decompressed = Vec::new();
//                 decoder.read_to_end(&mut decompressed)?;
//                 Ok(decompressed)
//             }
//             LpkEntryStorageType::Encrypted => {
//                 let database_key = Self::get_db_key(&self.file_name, aes_xor_key);
//                 let hex_string = hex::encode(database_key);
//                 let mut sha256 = Sha256::new();
//                 sha256.update(hex_string.as_bytes());
//                 let aes_key = sha256.finalize();

//                 // Assume CBC mode AES with a zero IV
//                 let iv = [0u8; 16];
//                 let cipher = Cbc::<Aes256, Pkcs7>::new_from_slices(&aes_key, &iv).unwrap();

//                 let decrypted = cipher.decrypt_vec(&bytes).unwrap();

//                 Ok(decrypted)
//             }
//         }
//     }

//     pub fn format_bytes(bytes: u64) -> String {
//         let sizes = ["B", "KB", "MB", "GB", "TB"];
//         let mut len = bytes as f64;
//         let mut order = 0;
//         while len >= 1024.0 && order < sizes.len() - 1 {
//             order += 1;
//             len /= 1024.0;
//         }
//         format!("{:.2}{}", len, sizes[order])
//     }
// }

// impl fmt::Display for LpkEntry {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(
//             f,
//             "FileName={}:ContentType={:?}:Size={}:StorageType={:?}",
//             self.file_name,
//             self.content_type,
//             Self::format_bytes(self.length as u64),
//             self.storage_type
//         )
//     }
// }

// pub struct GlobalConfiguration;

// impl GlobalConfiguration {
//     pub const ENTRY_SIZE: u32 = 128; // Replace with your actual entry size
// }
