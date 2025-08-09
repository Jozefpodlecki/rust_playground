use anyhow::Result;
use capstone::arch::x86::{X86Insn, X86OperandType, X86Reg};
use ringbuffer::{AllocRingBuffer, RingBuffer};

use crate::decompiler::types::Instruction;

pub struct Analyser {
    ring_buffer: AllocRingBuffer<Instruction>
}

impl Analyser {
    pub fn new() -> Self {
        let ring_buffer = AllocRingBuffer::new(5);

        Self {
            ring_buffer
        }
    }

    pub fn process_batch(&mut self, batch: Vec<Instruction>) -> Result<()> {

        for instruction in batch {
            self.ring_buffer.enqueue(instruction);

            // self.detect_function_prologues()
        }

        Ok(())
    }

    fn detect_function_prologues(&self) -> Option<u64> {
        let buffer: Vec<_> = self.ring_buffer.to_vec();

        if buffer.len() < 2 {
            return None;
        }

        let instr1 = &buffer[buffer.len() - 2];
        let instr2 = &buffer[buffer.len() - 1];

        if instr1.id == X86Insn::X86_INS_PUSH
            && instr2.id == X86Insn::X86_INS_MOV
        {
            if instr2.operands.len() == 2 {
                let operands = (&instr2.operands[0].op_type, &instr2.operands[1].op_type);
                 if let (X86OperandType::Reg(dest), X86OperandType::Reg(src)) = operands {
                    if dest.0 == X86Reg::X86_REG_RBP as u16 && src.0 == X86Reg::X86_REG_RSP as u16 {
                        return Some(instr1.address);
                    }
                }
            }
        }

        None
    }
}