use std::{collections::VecDeque, io::Read, vec};
use capstone::{arch::{self, x86::X86Insn, BuildsCapstone, BuildsCapstoneSyntax}, Capstone};
use anyhow::*;

use crate::disassembler::types::Instruction;

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

pub struct DisasmStream<R: Read> {
    reader: R,
    cs: Capstone,
    buffer: Vec<u8>,
    leftover: Vec<u8>,
    addr: u64,
    total_instr_len: u64,
    total_instr_count: u64,
    default_ratio: u64,
    items: VecDeque<Instruction>
}

impl<R: Read> DisasmStream<R> {
    pub fn new(reader: R, addr: u64, buf_size: usize) -> Result<Self> {

        let cs = build_capstone()?;

        Ok(Self {
            reader,
            cs,
            buffer: vec![0; buf_size],
            leftover: Vec::new(),
            addr,
            total_instr_len: 0,
            total_instr_count: 0,
            default_ratio: 7,
            items: VecDeque::with_capacity(1000)
        })
    }

    fn next_inner(&mut self) -> Result<()> {
        let bytes_read = self.reader.read(&mut self.buffer)?;
        if bytes_read == 0 && self.leftover.is_empty() {
            return Ok(())
        }

        let mut combined = std::mem::take(&mut self.leftover);
        combined.extend_from_slice(&self.buffer[..bytes_read]);

        let combined_len = combined.len() as u64;
        let ratio = if self.total_instr_count > 0 {
            (self.total_instr_len as f64 / self.total_instr_count as f64).ceil() as u64
        } else {
            self.default_ratio
        };

        let count = (combined_len / ratio.max(1)).max(1);
        let items = self.cs.disasm_count(&combined, self.addr, count as usize)?;
        let mut consumed = 0;

        for instr in items.into_iter() {

            let id = instr.id().0;
            let len = instr.len();
            consumed += len;
            self.addr += len as u64;
            self.total_instr_len += len as u64;
            self.total_instr_count += 1;

            if id == 0 {
                self.items.push_back(Instruction::invalid(X86Insn::from(id), &instr));
                continue;
            }

            self.items.push_back((instr, &self.cs).into());
        }

        if consumed < combined.len() {
            let leftover_len = combined.len() - consumed;
            combined.copy_within(consumed.., 0);
            combined.truncate(leftover_len);
            self.leftover = combined;
        }

        Ok(())
    }
}

impl<R: Read> Iterator for DisasmStream<R> {
    type Item = Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        
        if self.items.is_empty() {
            self.next_inner().expect("An error occurred whilst reading instructions");
        }

        self.items.pop_front()
    }
}