use std::{collections::HashMap, env, fs::{self, File}, io::{BufWriter, Cursor, Read, Seek, SeekFrom, Write}, path::{Path, PathBuf}};

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
                    keywords,
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

fn read_struct(cursor: &mut Cursor<Vec<u8>>) -> Result<String> {
    let name = read_string(cursor)?;
    println!("struct: {}", name);
    Ok(name)
}

fn read_field(cursor: &mut Cursor<Vec<u8>>) -> Result<(String, Vec<u8>)> {
    let name = read_string(cursor)?;
    let mut buffer = vec![0u8; 4];
    cursor.read_exact(&mut buffer)?;
    println!("field: {} {:?}", name, buffer);
    Ok((name, buffer))
}

fn read_field_until(cursor: &mut Cursor<Vec<u8>>, until: u8) -> Result<(String, Vec<u8>)> {
    let name = read_string(cursor)?;
    let mut buffer = vec![0u8; 4];
    cursor.read_exact(&mut buffer)?;
    
    let mut buffer = Vec::new();
    let mut byte = [0u8; 1];

    while let std::result::Result::Ok(_) = cursor.read_exact(&mut byte) {
        if byte[0] == until {
            break;
        }
        buffer.push(byte[0]);
    }
    buffer.truncate(buffer.len().saturating_sub(5));
    cursor.set_position(cursor.position() - 5);
    println!("field: {} {:?}", name, buffer);
    Ok((name, buffer))
}

fn read_field_n(cursor: &mut Cursor<Vec<u8>>, length: usize) -> Result<(String, Vec<u8>)> {
    let name = read_string(cursor)?;
    let mut buffer = vec![0u8; length];
    cursor.read_exact(&mut buffer)?;
    println!("field: {} {:?}", name, buffer);
    Ok((name, buffer))
}

pub fn parse_loa_data_test(path: &Path) -> Result<(String, String)> {
    let mut file = File::open(path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    let mut cursor: Cursor<Vec<u8>> = Cursor::new(data);
    
    let version = cursor.read_f32::<LittleEndian>()?;
    cursor.read_u32::<LittleEndian>()?;
    let object_id = cursor.read_u32::<LittleEndian>()?;
    let root_name = read_string(&mut cursor)?;

    let mut map: HashMap<String, String> = HashMap::new();

    let json_value: Value = serde_json::to_value(&map).unwrap();
    let json_str = json_value.to_string();

    let (name, value) = read_field(&mut cursor)?;
    let struct_name = read_struct(&mut cursor)?;
    let struct_name = read_struct(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let struct_name = read_struct(&mut cursor)?;
    let (name, value) = read_field_n(&mut cursor, 14)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let struct_name = read_struct(&mut cursor)?;
    let struct_name = read_struct(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let struct_name = read_struct(&mut cursor)?;
    let struct_name = read_struct(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let struct_name = read_struct(&mut cursor)?;
    let struct_name = read_struct(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let struct_name = read_struct(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let struct_name = read_struct(&mut cursor)?;
    let (name, value) = read_field_until(&mut cursor, b'e')?;
    let (name, value) = read_field(&mut cursor)?;
    let struct_name = read_struct(&mut cursor)?;
    let struct_name = read_struct(&mut cursor)?;
    // let (name, value) = read_field_until(&mut cursor, b'V')?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;
    let struct_name = read_struct(&mut cursor)?;
    let struct_name = read_struct(&mut cursor)?;
    let (name, value) = read_field(&mut cursor)?;

    Ok((root_name, json_str))
}

fn parse_loa_data(path: &Path) -> Result<(String, u32, Vec<String>)> {

    let mut file = File::open(path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    let data_length = data.len();
    let mut cursor: Cursor<Vec<u8>> = Cursor::new(data);
    
    let version = cursor.read_f32::<LittleEndian>()?;
    cursor.read_u32::<LittleEndian>()?;
    let object_id = cursor.read_u32::<LittleEndian>()?;
    let root_name = read_string(&mut cursor)?;

    let mut results = Vec::new();

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
                    results.push(s.trim_end_matches('\0').to_string());
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

        let files = collect_loa_files(&output_path).unwrap();
        
        // let (name, object_id, data) = parse_loa_data(&file_path).map_err(|err| println!("{err}")).unwrap();

        // println!("{} {} {:?}", name, object_id, data);
    }
}