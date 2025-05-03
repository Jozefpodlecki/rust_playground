use std::{collections::HashMap, fs::File, io::{BufRead, BufReader, Read}, path::Path, thread::sleep, time::Duration};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;
use anyhow::{Ok, Result};
use log::*;
use simple_logger::SimpleLogger;
use utils::pause;

mod utils;

#[derive(Debug)]
struct Record {
    length: u32,
    header: [u8; 4],
    header_hex: String,
    payload: Vec<u8>,
}

fn hex_line_to_bytes(line: &str) -> Vec<u8> {
    line.split_whitespace()
        .filter_map(|b| u8::from_str_radix(b, 16).ok())
        .collect()
}

fn format_hex_with_spaces(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{:02X}", byte)).collect::<Vec<String>>().join(" ")
}


// fn parse_str_record(line: &str) -> Option<Record> {
//     let bytes = hex::decode(line.replace(" ", "")).ok()?;
//     let mut cursor = Cursor::new(&bytes);

//     let mut length = cursor.read_u32::<LittleEndian>().ok()?;
//     let mut header = [0u8; 4];
//     cursor.read_exact(&mut header).ok()?;

//     let payload = cursor.into_inner()[8..].to_vec();
//     length = length - 8;

//     if payload.len() == length as usize {
//         Some(Record { length, header, payload })
//     } else {
//         let header_hex = format_hex_with_spaces(&header);
//         println!("{} {} {}", header_hex, payload.len(), length);
//         None
//     }
// }

fn parse_record(buffer: &[u8]) -> Option<Record> {
    let hex_string = String::from_utf8_lossy(buffer);
    let bytes = hex::decode(&*hex_string).ok()?;
    let mut cursor = Cursor::new(bytes);

    let mut length = cursor.read_u32::<LittleEndian>().ok()?;
    let mut header = [0u8; 4];
    cursor.read_exact(&mut header).ok()?;

    let payload = cursor.into_inner()[8..].to_vec();
    length = length - 8;

    if payload.len() == length as usize {
        let header_hex = format_hex_with_spaces(&header);
        // let payload_hex = format_hex_with_spaces(&record.payload);
        Some(Record { length, header_hex, header, payload })
    } else {
        // let header_hex = format_hex_with_spaces(&header);
        // println!("{} {} {}", header_hex, payload.len(), length);
        None
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    
    SimpleLogger::new().env().init().unwrap();

    let path = Path::new("C:\\Users\\jozef\\Documents\\dump\\dump_20250501_092824.data");
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let mut buffer: Vec<u8> = Vec::new();
    let mut i = 0;
    let mut grouped: HashMap<String, Vec<Record>> = HashMap::new();

    while reader.read_until(b'\n', &mut buffer)? > 0 {
        buffer.retain(|&byte| !byte.is_ascii_whitespace());
       
        if let Some(record) = parse_record(&buffer) {
            
            let records = grouped.entry(record.header_hex.clone()).or_default();
            records.push(record);
            
            // println!(
            //     "Record {}: length = {}, header = {}, payload = {}",
            //     i + 1,
            //     record.length,
            //     header_hex,
            //     payload_hex
            // );
        }
       
        buffer.clear();
        i += 1;
    }

    for (key, records) in grouped.iter() {
        println!("{} {}", key, records.len());
    }

    // for (i, line) in reader.lines().enumerate() {
    //     let line = line?;

    //     if let Some(record) = parse_record(&line) {
    //         let header_hex = format_hex_with_spaces(&record.header);
    //         let payload_hex = format_hex_with_spaces(&record.payload);
    //         println!(
    //             "Record {}: length = {}, header = {}, payload = {}",
    //             i + 1,
    //             record.length,
    //             header_hex,
    //             payload_hex
    //         );
    //     } else {
    //         // eprintln!("Error parsing line {}: {}", i + 1, line);
    //     }
    //     sleep(Duration::from_secs(1));
    // }

    // pause();

    Ok(())
}