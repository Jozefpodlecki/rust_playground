use anyhow::{bail, Result};
use decompiler_lib::decompiler::types::InstructionType;

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
            if self.cpu.rip as usize >= self.cpu.memory.size() {
                println!("RIP out of range: {:#x}", self.cpu.rip);
                break;
            }

            let instruction = match self.decoder.decode_next() {
                Some(instruction) => instruction,
                None => return Ok(()),
            };

            self.cpu.handle(instruction);
        }

        Ok(())
    }
}