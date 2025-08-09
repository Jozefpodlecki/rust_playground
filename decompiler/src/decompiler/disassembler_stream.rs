use std::{env::var, io::{self, BufReader, Read}, vec};
use capstone::{arch::{self, x86::{X86Insn, X86Operand, X86OperandType, X86Reg}, BuildsCapstone, BuildsCapstoneSyntax, DetailsArchInsn}, Capstone, Insn, InsnGroupType::CS_GRP_JUMP, InsnId, Instructions};
use anyhow::*;
use crate::decompiler::types::Instruction;

pub struct DisasmStream<R: Read> {
    reader: BufReader<R>,
    cs: Capstone,
    buffer: Vec<u8>,
    leftover: Vec<u8>,
    addr: u64,
    total_instr_len: u64,
    total_instr_count: u64,
}

impl<R: Read> DisasmStream<R> {
    pub fn new(reader: R, buf_size: usize) -> Result<Self> {

        let mut cs = Capstone::new()
            .x86()
            .mode(arch::x86::ArchMode::Mode64)
            .syntax(arch::x86::ArchSyntax::Intel)
            .build()?;

        cs.set_skipdata(true)?;
        cs.set_detail(true)?;

        Ok(Self {
            reader: BufReader::new(reader),
            cs,
            buffer: vec![0; buf_size],
            leftover: Vec::new(),
            addr: 0,
            total_instr_len: 0,
            total_instr_count: 0,
        })
    }

    pub fn next_batch(&mut self) -> Result<Vec<Instruction>> {
        let bytes_read = self.reader.read(&mut self.buffer)?;
        if bytes_read == 0 && self.leftover.is_empty() {
            return Err(anyhow!("Empty"));
        }

        let mut combined = std::mem::take(&mut self.leftover);
        combined.extend_from_slice(&self.buffer[..bytes_read]);

        let combined_len = combined.len() as u64;
        let ratio = if self.total_instr_count > 0 {
            (self.total_instr_len as f64 / self.total_instr_count as f64).ceil() as u64
        } else {
            7
        };

        let count = combined_len / ratio.max(1);
        let items = self.cs.disasm_count(&combined, self.addr, count as usize)?;
        let mut batch = vec![];
        let mut consumed = 0;

        for instr in items.into_iter() {
            consumed += instr.len();
            self.addr += instr.len() as u64;
            self.total_instr_len += instr.len() as u64;
            self.total_instr_count += 1;
            let detail = self.cs.insn_detail(instr)?;
            let arch_detail = detail.arch_detail();
            let x86_detail = arch_detail.x86().unwrap();
            batch.push((instr, x86_detail).into());
        }

        if consumed < combined.len() {
            let leftover_len = combined.len() - consumed;
            combined.copy_within(consumed.., 0);
            combined.truncate(leftover_len);
            self.leftover = combined;
        }

        Ok(batch)
    }
}
