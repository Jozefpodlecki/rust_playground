use std::{fs::{self, File}, io::{BufWriter, Read, Write}, path::Path};

use anyhow::*;

pub fn is_folder_empty(path: &Path) -> Result<bool> {
    let mut entries = fs::read_dir(path)?;
    Ok(entries.next().is_none())
}

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

pub fn save_hex_dump_pretty_from_reader<P: AsRef<Path>>(mut reader: impl Read, output_path: P, bytes_per_line: usize) -> Result<()> {
    let mut writer = BufWriter::new(File::create(&output_path)
        .with_context(|| format!("Failed to create file {:?}", output_path.as_ref()))?);

    let mut data = Vec::new();
    reader.read_to_end(&mut data)?;

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

pub fn save_hex_dump_pretty<P: AsRef<Path>>(input_path: P, output_path: P, bytes_per_line: usize) -> Result<()> {
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