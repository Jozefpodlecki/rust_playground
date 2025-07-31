use std::{env, fs::{self, File}, io::{BufWriter, Cursor, Read, Seek, Write}, path::{Path, PathBuf}};

use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt};
use log::info;
use crate::types::{LpkEntryType, LpkInfo, RunArgs};


pub fn save_pretty_hex_dump<P: AsRef<Path>>(input_path: P, output_path: P, bytes_per_line: usize) -> Result<()> {
    let data = fs::read(&input_path)
        .with_context(|| format!("Failed to read file {:?}", input_path.as_ref()))?;

    let mut writer = BufWriter::new(File::create(&output_path)
        .with_context(|| format!("Failed to create file {:?}", output_path.as_ref()))?);

    for chunk in data.chunks(bytes_per_line) {
        // Print hex part
        for (i, byte) in chunk.iter().enumerate() {
            write!(writer, "{:02X}", byte)?;
            if i < chunk.len() - 1 {
                write!(writer, " ")?;
            }
        }

        // Pad if line is shorter than bytes_per_line
        let pad = bytes_per_line.saturating_sub(chunk.len());
        if pad > 0 {
            let pad_spaces = pad * 3 - 1;
            write!(writer, "{:width$}", "", width = pad_spaces)?;
        }

        // Add space before ASCII
        write!(writer, "  |")?;

        // Print ASCII part
        for byte in chunk {
            let c = *byte as char;
            if c.is_ascii_graphic() || c == ' ' {
                write!(writer, "{}", c)?;
            } else {
                write!(writer, ".")?;
            }
        }

        writeln!(writer, "|")?;
    }

    Ok(())
}

pub fn guess_it_is_a_field(cursor: &mut Cursor<Vec<u8>>, name: &str) -> bool {

    let original_pos = cursor.position();

      if let std::result::Result::Ok(len) = cursor.read_u32::<LittleEndian>() {
        if len > 0 && len < 1000 {
            let mut buffer = vec![0u8; len as usize];
            if cursor.read_exact(&mut buffer).is_ok() && std::str::from_utf8(&buffer).is_ok() {
                cursor.set_position(original_pos);
                return true;
            }
        }

        cursor.set_position(original_pos);
        if cursor.seek(std::io::SeekFrom::Current(4)).is_ok() {
            if let std::result::Result::Ok(len2) = cursor.read_u32::<LittleEndian>() {
                if len2 > 0 && len2 < 1000 {
                    let mut buffer = vec![0u8; len2 as usize];
                    if cursor.read_exact(&mut buffer).is_ok() && std::str::from_utf8(&buffer).is_ok() {
                        cursor.set_position(original_pos);
                        return true;
                    }
                }
            }
        }
    }

    cursor.set_position(original_pos);
    false
}

pub fn parse_ue3_object_1(args: RunArgs) -> Result<()> {

      let RunArgs {
        output_path,
        ..
    } = args;

    // let path = Path::new(&output_path).join("data1").join("998.loa");
    // let pretty_hex_path = Path::new("998_hex.txt").to_owned();
    // save_pretty_hex_dump(&path, &pretty_hex_path, 32)?;

    let path = Path::new(&output_path).join("data3").join("10001.loa");
    let pretty_hex_path = Path::new("10001_hex.txt").to_owned();
    save_pretty_hex_dump(&path, &pretty_hex_path, 32)?;

    let mut file = File::open(&path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    let mut cursor: Cursor<Vec<u8>> = Cursor::new(data);

    let float_val = cursor.read_f32::<LittleEndian>()?;
    println!("Version: {}", float_val);

    let field_count = cursor.read_u32::<LittleEndian>()?;
    println!("Field count: {}", field_count);

    let object_id = cursor.read_u32::<LittleEndian>()?;
    println!("Object Id: {}", object_id);

    let len = cursor.read_u32::<LittleEndian>()?;
    let mut buffer = vec![0u8; len as usize];
    cursor.read_exact(&mut buffer)?;
    let mut struct_name = String::from_utf8_lossy(&buffer).to_string();
    
    let mut has_read = false;
    let mut len = 0;
    let mut previous_struct_name: String = "".into();
    let mut property_name: String = "".into();
    let mut is_field = false;
    let mut it = 0;
    let total_len = cursor.get_ref().len();

    loop {
        if !can_read(&cursor, 4) {
            break;
        }

        let len = cursor.read_u32::<LittleEndian>()?;

        if len == 0 || len > 200 {
            println!("Skipping invalid length: {}", len);
            break;
        }

        if !can_read(&cursor, len as usize) {
            println!("Not enough bytes to read name of length {}", len);
            break;
        }

        let mut buffer = vec![0u8; len as usize];
        cursor.read_exact(&mut buffer)?;
        let string_candidate = String::from_utf8_lossy(&buffer);

        if string_candidate == "ArrayValue" {
            println!("Field: ArrayValue in Struct: {}", struct_name);
            continue;
        }

        if string_candidate.chars().all(|c| c.is_ascii_graphic() || c == '\0') {

            if guess_it_is_a_field(&mut cursor, &string_candidate) {
                let value = cursor.read_u32::<LittleEndian>()?;
                println!("Field: {} {}", string_candidate, value);
            }
            else {
                previous_struct_name = struct_name;
                struct_name = string_candidate.to_string();
                println!("Struct: {}", struct_name);
            }
            // previous_struct_name = struct_name.clone();
            // struct_name = string_candidate.to_string();
            
        } else {
            println!("Unknown or binary data, len: {}, bytes: {:?}", len, &buffer);
            break;
        }

    }

    // loop {
    //     is_field = false;

    //     if !has_read {
    //         len = cursor.read_u32::<LittleEndian>()?;
    //     }

    //     has_read = false;

    //     if len > 1 && len < 200 {
    //         let current_pos = cursor.position() as usize;
    //         let mut buffer = vec![0u8; len as usize];

    //         if current_pos + len as usize <= total_len {
    //             cursor.read_exact(&mut buffer)?;
    //         }
    //         else {
    //             println!("{}", len);
    //             return Ok(());
    //         }

    //         previous_struct_name = struct_name;
    //         struct_name = match String::from_utf8(buffer) {
    //             std::result::Result::Ok(value) => value,
    //             Err(_) => {
    //                 println!("err");
    //                 return Ok(());
    //             },
    //         };
    //     }

    //     if cursor.position() as usize == total_len {
    //         break;
    //     }

    //     len = cursor.read_u32::<LittleEndian>()?;
    //     has_read = true;

    //     if len == 0 || len == 1 || len > 200 {
    //         property_name = struct_name;
    //         struct_name = previous_struct_name.clone();
    //         is_field = true;
    //     }

    //     if is_field {
    //         println!("Struct {} Field {}", struct_name, property_name);
    //     }
    //     else {
    //         println!("Struct {}", struct_name);
    //     }

    //     it += 1;
    // }


    Ok(())
}

fn can_read(cursor: &Cursor<Vec<u8>>, bytes: usize) -> bool {
    (cursor.position() as usize + bytes) <= cursor.get_ref().len()
}

pub fn extract_lpk(args: RunArgs) -> Result<()> {
    let RunArgs {
        lpk_dir,
        cipher_key,
        aes_xor_key,
        output_path,
        ..
    } = args;

    let output_path = Path::new(&output_path);

    for entry in fs::read_dir(&lpk_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("lpk") {
            continue;
        }

        let lpk_name = path.file_stem().unwrap().to_str().unwrap();
        let output_path = output_path.join(lpk_name);
        info!("Creating directory {}", output_path.to_str().unwrap());
        fs::create_dir_all(&output_path)?;
        let file_path = path.to_str().unwrap();

        let mut lpk_info = LpkInfo::new(file_path, &cipher_key)?;

        for entry in lpk_info.entries.iter_mut() {
            match entry {
                LpkEntryType::BlowfishCompressed(entry) => {
                    let file_name = entry.file_name.to_owned();
                    let content = entry.get_content(&aes_xor_key)?;
                    save_to_disk(&file_name, &output_path, &content)?;
                },
                LpkEntryType::Aes256CbcEncrypted(entry) => {
                    let file_name = entry.file_name.to_owned();
                    let content = entry.get_content(&aes_xor_key)?;
                    save_to_disk(&file_name, &output_path, &content)?;
                },
            }
        }
    }

    Ok(())
}

pub fn save_to_disk(file_name: &str, output_path: &Path, content: &[u8]) -> Result<()> {
    let output_path = output_path.join(file_name);
    info!("Saving to {}", output_path.to_str().unwrap());

    let mut file = File::create(&output_path)?;
    file.write_all(&content)?;

    Ok(())
}

pub fn collect_lpk_paths<P: AsRef<Path>>(lpk_dir: P) -> Result<Vec<PathBuf>> {
    let mut lpk_paths = Vec::new();

    for entry in fs::read_dir(lpk_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("lpk") {
            lpk_paths.push(path);
        }
    }

    Ok(lpk_paths)
}

fn clean(source: &str) -> String {
    let mut out_str = String::new();
    let source = source.to_uppercase();
    let key_table = [
        ("QP", 'Q', 0), ("QD", 'Q', 1), ("QW", 'Q', 2), ("Q4", 'Q', 3),
        ("QL", '-', 0), ("QB", '-', 1), ("QO", '-', 2), ("Q5", '-', 3),
        ("QC", '_', 0), ("QN", '_', 1), ("QT", '_', 2), ("Q9", '_', 3),
        ("XU", 'X', 0), ("XN", 'X', 1), ("XH", 'X', 2), ("X3", 'X', 3),
        ("XW", '!', 0), ("XS", '!', 1), ("XZ", '!', 2), ("X0", '!', 3),
    ];

    let mut i = 0;
    let chars = source.chars().collect::<Vec<_>>();

    while i < chars.len() {
        let rest: String = chars[i..].iter().collect();
        let subst = key_table.iter().find(|(key, _, pos)| {
            rest.starts_with(*key) && (i % 4 == *pos)
        });

        if let Some((_, replacement, key_len)) = subst {
            out_str.push(*replacement);
            i += 2;
        } else {
            out_str.push(chars[i]);
            i += 1;
        }
    }

    out_str
}

pub fn decrypt(source: &str) -> String {
    let source = source.to_uppercase();
    let mut out_str = String::new();

    for c in source.chars() {
        let mut x = c as i32;

        if c >= '0' && c <= '9' {
            x += 43;
        }

        let len = source.len() as i32;
        let mut i = (31 * (x - len - 65) % 36 + 36) % 36 + 65;
        if i >= 91 {
            i -= 43;
        }

        out_str.push(i as u8 as char);
    }

    let cleaned = clean(&out_str);
    cleaned.split('!').next().unwrap_or("").to_string()
}