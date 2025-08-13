use std::{fs::File, io::BufWriter};

use anyhow::{bail, Result};
use decompiler_lib::decompiler::Disassembler;
use log::info;
use std::io::Write;

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

    pub fn dump(self) -> Result<()> {

        let file = File::create("dump.txt")?;
        let mut writer = BufWriter::new(file);

        let bus = self.cpu.bus.borrow();
        let region = bus.find_region(0x1475c3043)?;
        let disassembler = Disassembler::from_memory(&region.data, region.start_addr, 1000)?;

        let stream = disassembler.disasm_all()?;

        for instr in stream {
            writeln!(writer, "{}", instr)?;
        }

        let region = bus.find_region(0x1469ea014)?;
        let disassembler = Disassembler::from_memory(&region.data, region.start_addr, 1000)?;
        let stream = disassembler.disasm_all()?;

        for instr in stream {
            writeln!(writer, "{}", instr)?;
        }

        writeln!(writer, "")?;
        writeln!(writer, "Registers:")?;
        writeln!(writer, "RAX: {:#X}", self.cpu.registers.rax)?;
        writeln!(writer, "RBX: {:#X}", self.cpu.registers.rbx)?;
        writeln!(writer, "RCX: {:#X}", self.cpu.registers.rcx)?;
        writeln!(writer, "RDX: {:#X}", self.cpu.registers.rdx)?;
        writeln!(writer, "RSP: {:#X}", self.cpu.registers.rsp)?;
        writeln!(writer, "RBP: {:#X}", self.cpu.registers.rbp)?;
        writeln!(writer, "RSI: {:#X}", self.cpu.registers.rsi)?;
        writeln!(writer, "RDI: {:#X}", self.cpu.registers.rdi)?;
        writeln!(writer, "R8 : {:#X}", self.cpu.registers.r8)?;
        writeln!(writer, "R9 : {:#X}", self.cpu.registers.r9)?;
        writeln!(writer, "R10: {:#X}", self.cpu.registers.r10)?;
        writeln!(writer, "R11: {:#X}", self.cpu.registers.r11)?;
        writeln!(writer, "R12: {:#X}", self.cpu.registers.r12)?;
        writeln!(writer, "R13: {:#X}", self.cpu.registers.r13)?;
        writeln!(writer, "R14: {:#X}", self.cpu.registers.r14)?;
        writeln!(writer, "R15: {:#X}", self.cpu.registers.r15)?;
        writeln!(writer, "RIP: {:#X}", self.cpu.rip)?;
        writeln!(writer, "RFLAGS: {:#X}", self.cpu.rflags)?;

        Ok(())
    }
}