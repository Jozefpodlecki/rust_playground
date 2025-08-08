use std::{fs::File, io::{BufWriter, Write}, path::Path};
use anyhow::*;
use capstone::{
    arch::{
        self, x86::{X86OperandIterator, X86OperandType, X86Reg}, BuildsCapstone, BuildsCapstoneSyntax, DetailsArchInsn
    }, Capstone, InsnDetail, InsnGroupId, InsnGroupType::{CS_GRP_CALL, CS_GRP_JUMP}, InsnId, RegId
};
use capstone::arch::x86::X86Insn::{X86_INS_INT3, X86_INS_NOP};
use log::*;

use crate::decompiler::{cfg_builder::CfgBuilder, disassembler::Disassembler};

mod disassembler;
mod cfg_builder;

pub struct Decompiler {
    disassembler: Disassembler,
    cfg_builder: CfgBuilder
}

impl Decompiler {
    pub fn new() -> Self {
        Self {
            disassembler: Disassembler::new(),
            cfg_builder: CfgBuilder::new(),
        }
    }

    pub fn run(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        let function_entries = self.disassembler.find_function_entries(data)?;

        let cfg = self.cfg_builder.build(function_entries)?;


        Ok(vec![])
    }
}