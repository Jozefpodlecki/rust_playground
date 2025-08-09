use std::io::{Read, Write};

use capstone::{arch::{self, x86::{X86Insn, X86Operand, X86OperandType, X86Reg}, BuildsCapstone, BuildsCapstoneSyntax, DetailsArchInsn}, Capstone, Insn, InsnGroupType::CS_GRP_JUMP, InsnId, Instructions};
use anyhow::*;
use object::{Object, ObjectSection};
use ringbuffer::{AllocRingBuffer, RingBuffer};

use crate::decompiler::disassembler_stream::DisasmStream;

pub struct Disassembler {
    cs: Capstone
}

pub struct DisassemblerResult {
    
}

impl Disassembler {
    pub fn new() -> Result<Self> {
        let mut cs = Capstone::new()
            .x86()
            .mode(arch::x86::ArchMode::Mode64)
            .syntax(arch::x86::ArchSyntax::Intel)
            .build()?;

        let mut ring_buf = AllocRingBuffer::<(InsnId, u64, Vec<X86Operand>)>::new(5);
        cs.set_skipdata(true)?;
        cs.set_detail(true)?;

        Ok(Self {
            cs
        })
    }

    pub fn disasm<R: Read>(&self, reader: R, buf_size: usize) -> Result<DisasmStream<R>> {
        DisasmStream::new(reader, buf_size)
    }

    pub fn to_writer<W: Write>(&self, data: &[u8], mut writer: W) -> Result<()> {

        let instructions = self.cs.disasm_all(data, 0)?;

        for i in instructions.iter() {
            writeln!(writer, "{}", i)?;
        }

        Ok(())
    }
}