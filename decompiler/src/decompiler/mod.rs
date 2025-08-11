use std::{collections::{HashMap, HashSet, VecDeque}, fs::File, io::{BufReader, BufWriter, Cursor, Read, Write}, path::Path, sync::mpsc::{sync_channel, Receiver, SyncSender}, thread::{self, sleep}, time::Duration};
use anyhow::{anyhow, Error, Result};
use capstone::arch::x86::X86Insn;
use log::*;
use object::ObjectSection;

use crate::decompiler::{loader::Loader, types::{CallTarget, Instruction}};

pub mod disassembler;
pub mod stream;
pub mod analyser;
pub mod types;
pub mod loader;
pub mod utils;

pub use disassembler::Disassembler;

#[derive(Debug)]
pub struct BasicBlock {
    pub start_addr: u64,
    pub instructions: Vec<Instruction>,
    pub successors: Vec<u64>,
}

#[derive(Debug)]
pub struct Function {
    pub start_addr: u64,
    pub basic_blocks: HashMap<u64, BasicBlock>,
}

pub struct Decompiler {
    functions: HashMap<u64, Function>,
    visited: HashSet<u64>,
}

impl Decompiler {
    pub fn new() -> Result<Self> {
        Ok(Self {
            functions: HashMap::new(),
            visited: HashSet::new(),
        })
    }

    pub fn run(&mut self, file: File, addr: u64) -> Result<()> {
        let mut seeds = VecDeque::new();
        seeds.push_back(addr);

        while let Some(current_addr) = seeds.pop_front() {
            if self.visited.contains(&current_addr) {
                continue;
            }
            self.visited.insert(current_addr);

            info!("Functions: {}", self.functions.len());
            // info!("Processing function or BB at 0x{:X}", current_addr);

            let file = file.try_clone()?;
            let disassembler = Disassembler::from_file(file, current_addr, 1000)?;
            let stream = disassembler.disasm_from_addr(current_addr)?;

            let mut basic_block = BasicBlock {
                start_addr: current_addr,
                instructions: vec![],
                successors: vec![],
            };

            for instruction in stream {
                basic_block.instructions.push(instruction.clone());

                match &instruction.kind {
                    types::InstructionType::ConditionalJump(condition_code, target_addr) => {
                        // Current BB successors: branch target + fall-through
                        basic_block.successors.push(*target_addr);
                        let fall_through = instruction.address + instruction.length;
                        basic_block.successors.push(fall_through);

                        // Enqueue successors for exploration
                        seeds.push_back(*target_addr);
                        seeds.push_back(fall_through);

                        break;
                    }
                    types::InstructionType::UnconditionalJump(target) => {
                        match target {
                            CallTarget::Direct(target_addr) => {
                                basic_block.successors.push(*target_addr);
                                seeds.push_back(*target_addr);
                            }
                            CallTarget::Indirect(_) | CallTarget::Memory { .. } => {
                                info!("Indirect jump at 0x{:X}", instruction.address);
                            }
                        }
                        break; // BB ends here
                    }
                    types::InstructionType::Call(target) => {
                        match target {
                            CallTarget::Direct(target_addr) => {
                                seeds.push_back(*target_addr); // treat calls as new functions
                            }
                            CallTarget::Indirect(_) | CallTarget::Memory { .. } => {
                                info!("Indirect call at 0x{:X}", instruction.address);
                            }
                        }
                        // Calls do not end BB (fall-through continues)
                    }
                    types::InstructionType::Ret => {
                        // BB ends here, no successors
                        break;
                    }
                    _ => {
                        // Continue accumulating instructions
                    }
                }
            }

            if !self.functions.contains_key(&current_addr) {
                self.functions.insert(
                    current_addr,
                    Function {
                        start_addr: current_addr,
                        basic_blocks: HashMap::new(),
                    },
                );
            }

            let func = self.functions.get_mut(&current_addr).unwrap();
            func.basic_blocks.insert(basic_block.start_addr, basic_block);
        }

        Ok(())
    }

}

fn test() -> Result<()> {


    Ok(())
}