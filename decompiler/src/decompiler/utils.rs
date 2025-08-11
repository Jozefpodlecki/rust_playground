use std::{fmt::format, fs::{self, File}, io::{BufReader, BufWriter, Write}, path::PathBuf};
use anyhow::*;
use crate::decompiler::Disassembler;

pub fn extract_asm_text_to_addr(input_path: PathBuf, end_addr: u64) -> Result<()> {
    
    let file_name = input_path.file_stem().unwrap().to_string_lossy();
    let addr = file_name.parse::<u64>()?;
    let output_path = format!("0x{:X}_0x{:X}.txt", addr, end_addr);
    
    let mut file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);

    let file = File::open(input_path)?;
    let disassembler = Disassembler::from_file(file, addr, 1000)?;
    
    let instructions = disassembler.disasm_to_addr(end_addr)?;

    for instruction in instructions {
        writeln!(writer, "{}", instruction)?;
    }

    Ok(())
}

pub fn extract_asm_text(input_path: PathBuf) -> Result<()> {

    let file_name = input_path.file_stem().unwrap().to_string_lossy();
    let addr = file_name.parse::<u64>()?;
    let output_path = format!("0x{:X}.txt", addr);
    
    let mut file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);

    let file = File::open(input_path)?;
    let disassembler = Disassembler::from_file(file, addr, 1000)?;
    
    let instructions = disassembler.disasm_all()?;

    for instruction in instructions {
        writeln!(writer, "{}", instruction)?;
    }

    Ok(())
}