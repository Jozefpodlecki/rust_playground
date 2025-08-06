use std::{collections::{HashMap, HashSet}, env, fs::{self, File}, io::{BufWriter, Cursor, Read, Seek, SeekFrom, Write}, path::{Path, PathBuf}};

use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt};
use log::info;
use serde_json::{json, Value};
use walkdir::WalkDir;
use crate::{lpk::LpkInfo, types::RunArgs};

#[derive(Debug)]
pub struct LoaFile {
    pub id: i32,
    pub relative_path: String,
    pub name: String,
    pub keywords: Vec<String>,
}

pub struct LoaFileIterator {
    id: i32,
    base_path: PathBuf,
    entries: walkdir::IntoIter,
}

impl LoaFileIterator {
    pub fn new(base_path: &Path) -> Self {
        Self {
            id: 1,
            base_path: base_path.to_path_buf(),
            entries: WalkDir::new(base_path).into_iter(),
        }
    }
}

impl Iterator for LoaFileIterator {
    type Item = Result<LoaFile>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

pub fn collect_loa_files(base_path: &Path) -> Result<Vec<LoaFile>> {
    let mut result = Vec::new();
    let mut id = 1;

    for entry in WalkDir::new(base_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();

        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if file_name.ends_with(".loa") {
                let relative_path = path.strip_prefix(base_path)?.to_string_lossy().to_string();
                let (name, object_id, keywords) = parse_loa_data(&path)?;

                result.push(LoaFile {
                    id,
                    relative_path,
                    name,
                    keywords: keywords.into_iter().collect(),
                });
                id += 1;
            }
        }
    }

    Ok(result)
}

fn read_string(cursor: &mut Cursor<Vec<u8>>) -> Result<String> {

    let len= cursor.read_u32::<LittleEndian>()? as usize;
    let mut buffer = vec![0u8; len];
    cursor.read_exact(&mut buffer)?;
    let mut value: String = String::from_utf8_lossy(&buffer).into();
    value = value.trim_end_matches('\0').to_string();
    Ok(value)
}

fn parse_loa_data(path: &Path) -> Result<(String, u32, HashSet<String>)> {

    let mut file = File::open(path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    let data_length = data.len();
    let mut cursor: Cursor<Vec<u8>> = Cursor::new(data);
    
    let version = cursor.read_f32::<LittleEndian>()?;
    cursor.read_u32::<LittleEndian>()?;
    let object_id = cursor.read_u32::<LittleEndian>()?;
    let root_name = read_string(&mut cursor)?;

    let mut results = HashSet::new();

    while let std::result::Result::Ok(pos) = cursor.seek(SeekFrom::Current(0)) {

        if (cursor.position() as usize + 5) > data_length {
            break;
        }

        let mut buf = [0u8; 5];
        if cursor.read_exact(&mut buf).is_err() {
            break;
        }

        let str_len = buf[0] as usize;

        if str_len >= 2 && buf[1] == 0 && buf[2] == 0 && buf[3] == 0 && is_ascii(buf[4]) {
            cursor.seek(SeekFrom::Current(-1)).unwrap();

            if (cursor.position() as usize + str_len) > data_length {
                break;
            }

            let mut str_buf = vec![0u8; str_len];
            if cursor.read_exact(&mut str_buf).is_ok() {
                if let std::result::Result::Ok(s) = String::from_utf8(str_buf.clone()) {
                    results.insert(s.trim_end_matches('\0').to_string());

                }
            }
        } else {
            cursor.seek(SeekFrom::Start(pos + 1)).unwrap();
        }
    }

    Ok((root_name, object_id, results))
}

fn is_ascii(byte: u8) -> bool {
    byte.is_ascii_graphic() || byte == b' '
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_loa_data() {
        let output_path = std::env::current_dir().unwrap();
        let output_path = output_path.join(r"target\debug");
        let file_path = output_path.join(r"data1\Common_Extra\XMLData\NPCFunction\10008.loa");
        let file_path = output_path.join(r"data4\EFGame_Extra\ClientData\XmlData\LookInfo\Human\EFDLChar_250621.whk_F1.loa");
        
        // let files = collect_loa_files(&output_path).unwrap();
        

        let (name, object_id, data) = parse_loa_data(&file_path).map_err(|err| println!("{err}")).unwrap();

        println!("{} {} {:?}", name, object_id, data);
    }
}