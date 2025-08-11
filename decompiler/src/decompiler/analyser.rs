use std::collections::{HashMap, HashSet};

use anyhow::Result;
use capstone::arch::x86::{X86Insn, X86OperandType, X86Reg};
use log::info;
use ringbuffer::{AllocRingBuffer, RingBuffer};

use crate::decompiler::types::Instruction;

pub struct Function {
    pub start_addr: u64,
    pub successors: HashMap<u64, BasicBlock>,
    pub source: ResolvedBy,
    pub needs_verification: bool,
}

pub struct BasicBlock {
    pub parent_addr: u64,
    pub start_addr: u64,
    pub successors: Vec<u64>,
}

#[derive(Debug)]
pub enum ResolvedBy {
    Main,
    PushMov,
    PushSubRsp,
    CallTarget, 
}

pub struct Analyser {
    ring_buffer: AllocRingBuffer<Instruction>,
    functions: HashMap<u64, Function>,
    call_targets: HashSet<u64>,
}

impl Analyser {
    pub fn new() -> Self {
        let ring_buffer = AllocRingBuffer::new(5);

        Self {
            ring_buffer,
            functions: HashMap::new(),
            call_targets: HashSet::new()
        }
    }

    pub fn setup(&mut self, addr: u64) -> Result<()> {
        self.functions.insert(
            addr,
            Function {
                start_addr: addr,
                successors: HashMap::new(),
                source: ResolvedBy::Main,
                needs_verification: false,
            },
        );

        Ok(())
    }

    pub fn second_verification(&mut self) -> Result<()> {
        let mut invalid_functions = vec![];

        for (addr, function) in self.functions.iter_mut() {
            
            if !function.needs_verification {
                continue;
            }
            
            if self.call_targets.contains(addr)  {
                function.needs_verification = false;
                // function.source += ResolvedBy::CallTarget;
                continue;
            }

            // invalid_functions.push(*addr);
        }

        for addr in invalid_functions {
            self.functions.remove(&addr);
        }

        Ok(())
    }

    pub fn process_batch(&mut self, batch: Vec<Instruction>) -> Result<()> {

        for instruction in batch {
            self.update_call_targets(&instruction);
            self.ring_buffer.enqueue(instruction);

            if let Some((addr, resolved_by, needs_verification)) = self.detect_function_prologues() {
              
                if self.functions.contains_key(&addr) {
                    continue; 
                }

                // info!("Found function prologue 0x{:X}", addr);

                self.functions.insert(
                    addr,
                    Function {
                        start_addr: addr,
                        successors: HashMap::new(),
                        source: resolved_by,
                        needs_verification,
                    },
                );
            }
        }

        Ok(())
    }

    
    fn update_call_targets(&mut self, inst: &Instruction) {
        if inst.id == X86Insn::X86_INS_CALL {
            if let X86OperandType::Imm(imm) = inst.operands[0].op_type {
                let target = imm as u64;
                self.call_targets.insert(target);
            }
        }
    }

    fn detect_function_prologues(&self) -> Option<(u64, ResolvedBy, bool)> {
        let buffer: Vec<_> = self.ring_buffer.to_vec();
        let n = buffer.len();
        
        if n < 2 {
            return None;
        }

        for i in 0..(n - 1) {
            let a = &buffer[i];
            let b = &buffer[i + 1];
            if a.id == X86Insn::X86_INS_PUSH && b.id == X86Insn::X86_INS_MOV {
                if let (Some(dest), Some(src)) = (b.operands.get(0), b.operands.get(1)) {
                    if let (X86OperandType::Reg(dest_r), X86OperandType::Reg(src_r)) =
                        (&dest.op_type, &src.op_type)
                    {
                        if dest_r.0 == X86Reg::X86_REG_RBP as u16
                            && src_r.0 == X86Reg::X86_REG_RSP as u16
                        {
                            return Some((a.address, ResolvedBy::PushMov, true));
                        }
                    }
                }
            }
            if a.id == X86Insn::X86_INS_PUSH && b.id == X86Insn::X86_INS_SUB {
                if let (Some(dst), Some(src)) = (b.operands.get(0), b.operands.get(1)) {
                    if let (X86OperandType::Reg(dst_r), X86OperandType::Imm(_)) =
                        (&dst.op_type, &src.op_type)
                    {
                        if dst_r.0 == X86Reg::X86_REG_RSP as u16 {
                            return Some((a.address, ResolvedBy::PushSubRsp, true));
                        }
                    }
                }
            }
        }

        None
    }
}