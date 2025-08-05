use std::{env, fs::{self, File}, io::{BufWriter, Cursor, Read, Seek, Write}, path::{Path, PathBuf}};

use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt};
use log::info;
use serde_json::{json, Value};

pub fn save_pretty_hex_dump_from_slice<P: AsRef<Path>>(data: Vec<u8>, output_path: P, bytes_per_line: usize) -> Result<()> {
    
    let mut writer = BufWriter::new(File::create(&output_path)
        .with_context(|| format!("Failed to create file {:?}", output_path.as_ref()))?);

    for chunk in data.chunks(bytes_per_line) {
        for (i, byte) in chunk.iter().enumerate() {
            write!(writer, "{:02X}", byte)?;
            if i < chunk.len() - 1 {
                write!(writer, " ")?;
            }
        }

        let pad = bytes_per_line.saturating_sub(chunk.len());
        if pad > 0 {
            let pad_spaces = pad * 3 - 1;
            write!(writer, "{:width$}", "", width = pad_spaces)?;
        }

        write!(writer, "  |")?;

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

pub fn save_pretty_hex_dump<P: AsRef<Path>>(input_path: P, output_path: P, bytes_per_line: usize) -> Result<()> {
    let data = fs::read(&input_path)
        .with_context(|| format!("Failed to read file {:?}", input_path.as_ref()))?;

    let mut writer = BufWriter::new(File::create(&output_path)
        .with_context(|| format!("Failed to create file {:?}", output_path.as_ref()))?);

    for chunk in data.chunks(bytes_per_line) {
        for (i, byte) in chunk.iter().enumerate() {
            write!(writer, "{:02X}", byte)?;
            if i < chunk.len() - 1 {
                write!(writer, " ")?;
            }
        }

        let pad = bytes_per_line.saturating_sub(chunk.len());
        if pad > 0 {
            let pad_spaces = pad * 3 - 1;
            write!(writer, "{:width$}", "", width = pad_spaces)?;
        }

        write!(writer, "  |")?;

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