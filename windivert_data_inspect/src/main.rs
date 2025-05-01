use std::{fs::File, io::{BufRead, BufReader, Read}, path::Path};
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


fn parse_record(line: &str) -> Option<Record> {
    let bytes = hex::decode(line.replace(" ", "")).ok()?;
    let mut cursor = Cursor::new(&bytes);

    let mut length = cursor.read_u32::<LittleEndian>().ok()?;
    let mut header = [0u8; 4];
    cursor.read_exact(&mut header).ok()?;

    let payload = cursor.into_inner()[8..].to_vec();
    length = length - 8;

    if payload.len() == length as usize {
        Some(Record { length, header, payload })
    } else {
        let header_hex = format_hex_with_spaces(&header);
        println!("{} {} {}", header_hex, payload.len(), length);
        None
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    
    SimpleLogger::new().env().init().unwrap();

    // let test = hex::decode("20 00 00 00 55 73 00 01 18 52 90 05 03 E7 7F 0C C0 D6 C2 A7 A5 45 B5 42 D4 FE F8 B1 0A 84 2D 0E".replace(" ", "")).unwrap();

    let path = Path::new("C:\\Users\\jozef\\Documents\\dump\\dump_20250501_092824.data");
    let file = File::open(path)?;
    let reader = BufReader::new(file);


    for (i, line) in reader.lines().enumerate() {
        let line = line?;

        if let Some(record) = parse_record(&line) {
            let header_hex = format_hex_with_spaces(&record.header);
            let payload_hex = format_hex_with_spaces(&record.payload);
            println!(
                "Record {}: length = {}, header = {}, payload = {}",
                i + 1,
                record.length,
                header_hex,
                payload_hex
            );
        } else {
            // eprintln!("Error parsing line {}: {}", i + 1, line);
        }
    }

    pause();

    Ok(())
}