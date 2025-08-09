use anyhow::*;
use capstone::{arch::x86::X86Insn, Capstone, Insn};
use std::collections::{HashMap, HashSet, VecDeque};

use crate::decompiler::types::Instruction;

pub struct BasicBlock {
    pub start_addr: u64,
    pub end_addr: u64,
    pub instructions: Vec<u64>,
    pub successors: Vec<u64>,
}

pub struct Cfg {
    pub blocks: HashMap<u64, BasicBlock>,
}

pub struct CfgBuilder {
    cs: Capstone,
    base_addr: u64,
}

impl CfgBuilder {
    pub fn new(cs: Capstone, base_addr: u64) -> Self {
        Self { cs, base_addr }
    }

    pub fn build(&mut self, entry_points: Vec<u64>) -> Result<Cfg> {
        let mut cfg = Cfg { blocks: HashMap::new() };
        let mut to_process: VecDeque<u64> = VecDeque::from(entry_points);
        let mut processed: HashSet<u64> = HashSet::new();

        // while let Some(addr) = to_process.pop_front() {
        //     if processed.contains(&addr) || addr < self.base_addr || addr >= self.base_addr + self.data.len() as u64 {
        //         continue;
        //     }

        //     let block = self.build_basic_block(addr)?;
        //     for &succ in &block.successors {
        //         if !processed.contains(&succ) {
        //             to_process.push_back(succ);
        //         }
        //     }

        //     processed.insert(addr);
        //     cfg.blocks.insert(addr, block);
        // }

        Ok(cfg)
    }

    fn build_basic_block(&self, start_addr: u64) -> Result<BasicBlock> {
        let mut instructions: Vec<u64> = Vec::new();
        let mut successors = Vec::new();
        let mut offset: usize = (start_addr - self.base_addr) as usize;
        let mut current_addr = start_addr;

        loop {
            // if offset >= self.data.len() {
            //     break;
            // }

            // let insns = self.cs.disasm_count(&self.data[offset..], current_addr, 1)?;
            // if insns.is_empty() {
            //     break;
            // }

            // let insn = &insns[0];
            // instructions.push(insn.clone());

            // let insn_len = insn.bytes().len() as u64;
            // let next_addr = current_addr + insn_len;


            // let id = insn.id();

            // if id == X86Insn::X86_INS_RET.into() {
            //     break;
            // }

            // else if id == X86Insn::X86_INS_JMP as u32 {
            //     if let Some(target) = self.get_jump_target(insn)? {
            //         successors.push(target);
            //     }
            //     break;
            // }

            // else if id.is_conditional_jump() {
            //     if let Some(target) = self.get_jump_target(insn)? {
            //         successors.push(target);
            //     }
            //     successors.push(next_addr); // fall-through
            //     break;
            // }

            // else if id == X86Insn::X86_INS_CALL as u32 {
            // }

            // current_addr = next_addr;
            // offset += insn_len as usize;
        }

        let end_addr = current_addr;
        Ok(BasicBlock {
            start_addr,
            end_addr,
            instructions: vec![],
            successors,
        })
    }

    fn get_jump_target(&self, insn: &Insn) -> Result<Option<u64>> {
        // let detail = self.cs.insn_detail(insn)?;
        // let arch_detail = detail.arch_detail();
        // let x86_detail = arch_detail.x86().ok_or_else(|| anyhow!("Not x86 instruction"))?;
        // let ops = x86_detail.operands();

        // if ops.len() == 1 {
        //     if let capstone::arch::x86::X86OperandType::Imm(imm) = ops[0].op_type {
        //         return Ok(Some(imm as u64));
        //     }
        // }

        Ok(None)
    }
}
