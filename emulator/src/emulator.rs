use anyhow::{bail, Result};
use decompiler_lib::decompiler::types::InstructionType;
use log::info;

use crate::{cpu::Cpu, decoder::Decoder};

pub struct Emulator {
    cpu: Cpu,
    decoder: Decoder,
}

impl Emulator {
    pub fn new(cpu: Cpu, decoder: Decoder) -> Self {
        Self { cpu, decoder }
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
          
            let instruction = self.decoder.decode_next(self.cpu.rip)?;

            info!("Instruction: {}", instruction);

            self.cpu.handle(instruction)?;
        }

        Ok(())
    }
}