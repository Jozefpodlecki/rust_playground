use decompiler_lib::decompiler::types::{Instruction, InstructionType, Operand};

use crate::{memory::Memory, registers::Registers};


#[derive(Debug, Default, Clone)]
pub struct Cpu {
    pub registers: Registers,
    pub rip: u64,
    pub rflags: u64,
    pub memory: Memory,
}


impl Cpu {
    pub fn new(mem_size: usize) -> Self {
        Self {
            registers: Registers::default(),
            rip: 0,
            rflags: 0,
            memory: Memory::new(mem_size),
        }
    }

    pub fn handle(&mut self, instruction: Instruction) {
        match instruction.kind {
            InstructionType::Invalid => {

            },
            InstructionType::Push(operand) => {
                match operand {
                    Operand::Reg(register) => {

                    },
                    Operand::Imm(addr) => {

                    },
                    Operand::Memory { base, index, disp, segment } => todo!(),
                }

            },
            InstructionType::UnconditionalJump(call_target) => {

            },
            InstructionType::Call(call_target) => {
                
            },
            InstructionType::Ret => {

            },
        }
    }
}