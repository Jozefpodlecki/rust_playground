use capstone::arch::x86::{X86Insn, X86Operand, X86OperandType};

use crate::decompiler::types::{ConditionCode, InstructionType, Operand, Register};

pub fn capstone_first_operand_to_internal(ops: &[X86Operand]) -> Operand {
    if let Some(op) = ops.get(0) {
        match &op.op_type {
            X86OperandType::Reg(reg) => Operand::Reg(Register::from(*reg)),
            X86OperandType::Imm(val) => Operand::Imm(*val),
            X86OperandType::Mem(mem) => Operand::Memory {
                base: if mem.base().0 == 0 { None } else { Some(Register::from(mem.base())) },
                index: {
                    let idx = mem.index();
                    if idx.0 == 0 {
                        None
                    } else {
                        Some((Register::from(idx), mem.scale() as u8))
                    }
                },
                disp: mem.disp(),
                size: op.size.into(),
                segment: if mem.segment().0 == 0 { None } else { Some(Register::from(mem.segment())) },
            },
            _ => Operand::Imm(0), // fallback for unknown types
        }
    } else {
        // fallback if the slice is empty
        Operand::Imm(0)
    }
}

pub fn get_operand(ops: &[X86Operand]) -> Operand {
    unsafe { ops.first().unwrap_unchecked().clone().into() }
}

pub fn get_2_operands(ops: &[X86Operand]) -> (Operand, Operand) {
    (ops[0].clone().into(), ops[1].clone().into())
}

pub fn capstone_operands_to_internal(ops: &[X86Operand]) -> Vec<Operand> {
    ops.iter().map(|op| {
        match &op.op_type {
            X86OperandType::Reg(reg) => Operand::Reg(Register::from(*reg)),
            X86OperandType::Imm(val) => Operand::Imm(*val),
            X86OperandType::Mem(mem) => Operand::Memory {
                base: if mem.base().0 == 0 { None } else { Some(Register::from(mem.base())) },
                index: {
                    let idx = mem.index();
                    if idx.0 == 0 {
                        None
                    } else {
                        Some((Register::from(idx), mem.scale() as u8))
                    }
                },
                disp: mem.disp(),
                size: op.size.into(),
                segment: if mem.segment().0 == 0 { None } else { Some(Register::from(mem.segment())) },
            },
            _ => Operand::Imm(0), // fallback, could consider Option or Result here instead
        }
    }).collect()
}

pub fn extract_target(operands: &[capstone::arch::x86::X86Operand]) -> Option<Operand> {
    if operands.len() != 1 {
        return None;
    }

    match &operands[0].op_type {
        capstone::arch::x86::X86OperandType::Imm(imm) => Some(Operand::Imm(*imm)),
        capstone::arch::x86::X86OperandType::Reg(reg) => Some(Operand::Reg(Register::from(*reg))),
        capstone::arch::x86::X86OperandType::Mem(mem) => Some(Operand::Memory {
            base: if mem.base().0 == 0 { None } else { Some(Register::from(mem.base())) },
            index: {
                let idx = mem.index();
                if idx.0 == 0 {
                    None
                } else {
                    Some((Register::from(idx), mem.scale() as u8))
                }
            },
            disp: mem.disp(),
            size: operands[0].size.into(),
            segment: if mem.segment().0 == 0 { None } else { Some(Register::from(mem.segment())) },
        }),
        _ => None,
    }
}