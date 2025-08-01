use std::{env, fs::{self, File}, io::{BufWriter, Cursor, Read, Seek, Write}, path::{Path, PathBuf}};

use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt};
use log::info;
use crate::{lpk::LpkInfo, types::RunArgs};

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