use std::{env, fs::File, io::{BufReader, BufWriter, Cursor, Read, Write}, path::{Path, PathBuf}, time::Duration};

use anyhow::*;
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use chrono::Local;
use hex::ToHex;
use log::*;

use crate::consumer::Consumer;


pub struct Processor {

}

impl Processor {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(&mut self, filter: String) -> Result<()> {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("dump_{}.data", timestamp);
        let path = Path::new(&filename);
        let abs_path = env::current_dir()?.join(path);

        let mut consumer = Consumer::new();
        let mut rx = consumer.start(filter.to_string()).await?; 
        let file = File::create(path)?;
        let mut writer: BufWriter<File> = BufWriter::new(file);
        info!("Output will be saved to: {}", abs_path.display());
    
        loop {
            tokio::select! {
                data = rx.recv() => {
                    if let Some(data) = data {
                        self.handle(&mut writer, data);
                    } else {
                        break;
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    info!("Received Ctrl+C, shutting down.");
                    // writer.flush()?;
                    break;
                }
            }
    
        }
    
        consumer.stop()?;

        Ok(())
    }

    pub fn handle(&mut self, writer: &mut BufWriter<File>, data: Vec<u8>) -> Result<()> {
        let mut cursor = Cursor::new(&data);
        let length = cursor.read_u32::<LittleEndian>().unwrap();
        let noise = cursor.read_u64::<LittleEndian>().ok();

        let message_type = LittleEndian::read_u32(&data[4..8]);

        if message_type == 0x0102EA16 {

            let spaced_hex: String = data[8..]
                .iter().encode_hex_upper::<String>()
                .as_bytes()
                .chunks(2)
                .map(std::str::from_utf8)
                .filter_map(Result::ok)
                .collect::<Vec<&str>>()
                .join(" ");

            write!(writer, "{} {}\r\n", length, spaced_hex)?;
            println!("chat {}\n", spaced_hex);

            return Ok(())
        }

        if let Some(unknown_id) = noise {
            match unknown_id {
                0x546781DB0103C5AC => return Ok(()),
                0x9742B50F01039962 => return Ok(()),
                0xB26ECED4010376CB => return Ok(()),
                0x0100A7660AB1FCC => return Ok(()),
                0x0103C5AC546781C3 => return Ok(()),
                0x0100914175EA5F4C => return Ok(()),
                0x0103439FE4327B03 => return Ok(()),
                0xA0B09ED101002E7B => return Ok(()),
                0xE4327B030103439F => return Ok(()),
                0x546781C30103C5AC => return Ok(()),
                0x75EA5F4C01009141 => return Ok(()),
                0x1EF3E88C010381DC => return Ok(()), // rare
                0x9BACCED40103AD3A => return Ok(()), // rare 600 bytes+
                0x1EF3E8A3010301DC => {
                    println!("song end?");
                    return Ok(())
                }
                0x919399180103182F => {
                    println!("song end");
                    return Ok(())
                }
                0x4BEF136F01034FB4 => {
                    println!("type on chat | song skill start");
                    return Ok(())
                },
                0xEF133B130103AEB3 => {
                    println!("type on chat | singing");
                }
                0x9399180103182F25 => {
                    println!("song cancel");
                    return Ok(());
                }
                // 0x75EA5F4C01009141 => return Ok(()),
                0x546781DA0103C5AC => {
                    let spaced_hex: String = data[12..]
                        .iter().encode_hex_upper::<String>()
                        .as_bytes()
                        .chunks(2)
                        .map(std::str::from_utf8)
                        .filter_map(Result::ok)
                        .collect::<Vec<&str>>()
                        .join(" ");

                    let x = cursor.read_u64::<LittleEndian>().unwrap();
                    let y = cursor.read_u64::<LittleEndian>().unwrap();
                    let z = cursor.read_u64::<LittleEndian>().unwrap();

                    println!("move {} {} {}", x,y,z);
                    return Ok(())
                },
                0x6488BC6701033A2B => {
                    let spaced_hex: String = data[12..]
                        .iter().encode_hex_upper::<String>()
                        .as_bytes()
                        .chunks(2)
                        .map(std::str::from_utf8)
                        .filter_map(Result::ok)
                        .collect::<Vec<&str>>()
                        .join(" ");

                    println!("stop {}", spaced_hex);
                    return Ok(())
                }
                _ => {}
            }
        }

        let spaced_hex: String = data[4..]
            .iter().encode_hex_upper::<String>()
            .as_bytes()
            .chunks(2)
            .map(std::str::from_utf8)
            .filter_map(Result::ok)
            .collect::<Vec<&str>>()
            .join(" ");

        write!(writer, "{} {}\r\n", length, spaced_hex)?;
        println!("{} {}\n", length, spaced_hex);

        Ok(())
    }
}