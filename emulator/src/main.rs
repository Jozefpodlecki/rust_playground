use std::{fs::File, path::PathBuf};

use flexi_logger::Logger;
use anyhow::Result;
use crate::{cpu::Cpu, decoder::Decoder, emulator::Emulator};
use decompiler_lib::decompiler::Disassembler;

mod cpu;
mod registers;
mod memory;
mod emulator;
mod decoder;
mod types;
fn main() -> Result<()> {
    Logger::try_with_str("info")?.start()?;

    let file_path = r"";

    let mut cpu = Cpu::new(1024 * 1024);
    let decoder = Decoder::new(&PathBuf::from(file_path))?;
    let mut emulator = Emulator::new(cpu, decoder);

    emulator.run()?;

    Ok(())
}
