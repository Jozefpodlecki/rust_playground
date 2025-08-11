#![allow(warnings)]

use std::{fmt::format, fs::{self, File}, io::{BufReader, BufWriter, Write}, path::PathBuf};
use anyhow::*;
use flexi_logger::Logger;
use crate::decompiler::{utils::*, Decompiler, Disassembler};

mod decompiler;
fn main() -> Result<()> {
    Logger::try_with_str("info")?.start()?;
    // let path = r"C:\repos\rust_playground\test_app.exe";
    // let content = fs::read(path)?;
    let path = r"C:\repos\rust_playground\140696078127104.data";
    let input_path = PathBuf::from(path);
    let file_name = input_path.file_stem().unwrap().to_string_lossy();
    let addr = file_name.parse::<u64>()?;

    // extract_asm_text_to_addr(path.into(), 0x7ff65bc210c7)?;
    // extract_asm_text(path.into())?;

    let file = File::open(path)?;
    let mut decompiler = Decompiler::new()?;
    let decompiled = decompiler.run(file, addr)?;

    Ok(())
}