use std::num::NonZeroUsize;

use capstone::{arch::{self, BuildsCapstone, BuildsCapstoneSyntax}, Capstone};

use anyhow::Result;
use decompiler_lib::decompiler::types::Instruction;
use lru::LruCache;

use crate::emulator::Bus;

fn build_capstone() -> Result<Capstone> {
    let mut cs = Capstone::new()
        .x86()
        .mode(arch::x86::ArchMode::Mode64)
        .syntax(arch::x86::ArchSyntax::Intel)
        .build()?;
    cs.set_skipdata(true)?;
    cs.set_detail(true)?;
    Ok(cs)
}

pub struct Decoder<'a> {
    bus: &'a Bus,
    cs: Capstone,
    cache: LruCache<u64, Instruction>,
}

impl<'a> Decoder<'a> {
    pub fn new(bus: &'a Bus) -> Result<Self> {
        let cs = build_capstone()?;

        Ok(Self { 
            bus,
            cs,
            cache: LruCache::new(NonZeroUsize::new(10).unwrap())
        })
    }

    pub fn decode_next(&mut self, rip: u64) -> Result<Instruction> {

        if let Some(instr) = self.cache.get(&rip) {
            return Ok(instr.to_owned())
        }

        let mut code = vec![0u8; 15];
        self.bus.read_exact(rip, &mut code)?;

        let instructions = self.cs.disasm_count(&code, rip, 1)?;

        let cs_insn = instructions.get(0)
            .ok_or_else(|| anyhow::anyhow!("Failed to decode instruction at {:#x}", rip))?;

        let instruction: Instruction = (cs_insn, &self.cs).into();
        self.cache.push(rip, instruction.clone());

        Ok(instruction)
    }

}