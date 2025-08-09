use std::{fs::File, io::{BufWriter, Cursor, Write}, path::Path};
use anyhow::Result;
use capstone::{
    arch::{
        self, x86::{X86OperandIterator, X86OperandType, X86Reg}, BuildsCapstone, BuildsCapstoneSyntax, DetailsArchInsn
    }, Capstone, InsnDetail, InsnGroupId, InsnGroupType::{CS_GRP_CALL, CS_GRP_JUMP}, InsnId, RegId
};
use capstone::arch::x86::X86Insn::{X86_INS_INT3, X86_INS_NOP};
use log::*;

use crate::decompiler::{analyser::Analyser, cfg_builder::CfgBuilder, loader::Loader};

mod disassembler;
mod disassembler_stream;
mod cfg_builder;
mod analyser;
mod types;
mod loader;

pub use disassembler::Disassembler;

pub struct Decompiler {
    disassembler: Disassembler,
    analyser: Analyser,
    loader: Loader,
    // cfg_builder: CfgBuilder
}

impl Decompiler {
    pub fn new() -> Result<Self> {
        Ok(Self {
            disassembler: Disassembler::new()?,
            analyser: Analyser::new(),
            loader: Loader::new()
        })
    }

    pub fn run(&mut self, data: &[u8]) -> Result<Vec<u8>> {
    
        let data = self.loader.get_text_section_data(data)?;
        let data = Cursor::new(data);

        let mut iterator = self.disassembler.disasm(data, 1000)?;

        while let Ok(batch) = iterator.next_batch() {
            self.analyser.process_batch(batch);
        }

        Ok(vec![])
    }
}