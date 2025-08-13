use decompiler_lib::decompiler::types::{ConditionCode, Operand, OperandSize, Register};
use anyhow::{bail, Result};
use crate::{bus::SharedBus, registers::Registers};

pub fn calc_address(
    registers: &Registers,
    base: Option<Register>,
    index: Option<(Register, u8)>,
    disp: i64
) -> u64 {
    let base_val = base.map_or(0, |r| registers.get(r));
    let index_val = index.map_or(0, |(r, scale)| registers.get(r) * scale as u64);
    base_val.wrapping_add(index_val).wrapping_add(disp as u64)
}

pub fn evaluate_condition(
    rcx: u64,
    rflags: u64,
    cond: ConditionCode) -> bool {
    let cf = (rflags >> 0) & 1 != 0;  // Carry Flag
    let pf = (rflags >> 2) & 1 != 0;  // Parity Flag
    let af = (rflags >> 4) & 1 != 0;  // Adjust Flag (optional)
    let zf = (rflags >> 6) & 1 != 0;  // Zero Flag
    let sf = (rflags >> 7) & 1 != 0;  // Sign Flag
    let of = (rflags >> 11) & 1 != 0; // Overflow Flag

    match cond {
        ConditionCode::Overflow => of,
        ConditionCode::NotOverflow => !of,
        ConditionCode::Below => cf,
        ConditionCode::AboveOrEqual => !cf,
        ConditionCode::Equal => zf,
        ConditionCode::NotEqual => !zf,
        ConditionCode::BelowOrEqual => cf || zf,
        ConditionCode::Above => !cf && !zf,
        ConditionCode::Sign => sf,
        ConditionCode::NotSign => !sf,
        ConditionCode::ParityEven => pf,
        ConditionCode::ParityOdd => !pf,
        ConditionCode::Less => sf != of,       // signed <
        ConditionCode::GreaterOrEqual => sf == of, // signed >=
        ConditionCode::LessOrEqual => zf || (sf != of), // signed <=
        ConditionCode::Greater => !zf && (sf == of),   // signed >
        ConditionCode::CXZ => rcx == 0, // RCX == 0
    }
}

pub fn get_count(registers: &Registers, operand: Operand) -> Result<u8> {
    let count = match operand {
        Operand::Imm(count) => count as u8,
        Operand::Reg(reg) => (registers.get(reg) & 0xFF) as u8,
        _ => anyhow::bail!("Invalid shift count operand"),
    };
    Ok(count)
}

pub fn read_operand_u64_rip(
    bus: &SharedBus,
    registers: &Registers,
    operand: Operand
) -> u64 {
    match operand {
        Operand::Imm(addr) => addr as u64,
        Operand::Reg(reg) => registers.get(reg),
        Operand::Memory { base, index, disp, size, segment } => {
            let base_val = base.map_or(0, |r| registers.get(r));
            let index_val = index.map_or(0, |(r, scale)| registers.get(r) * scale as u64);
            let addr = base_val.wrapping_add(index_val).wrapping_add(disp as u64);
            addr
        },
    }
}

pub fn read_operand_u64(
    bus: &SharedBus,
    registers: &Registers,
    op: Operand
) -> u64 {
    match op {
        Operand::Reg(reg) => registers.get(reg),
        Operand::Imm(val) => val as u64,
        Operand::Memory { base, index, disp, size, segment: _ } => {
            let bus = bus.borrow();
            let addr = calc_address(&registers, base, index, disp);
            match size {
                OperandSize::Byte => bus.read_u8(addr).unwrap() as u64,
                OperandSize::Word => bus.read_u16(addr).unwrap() as u64,
                OperandSize::Dword => bus.read_u32(addr).unwrap() as u64,
                OperandSize::Qword => bus.read_u64(addr).unwrap(),
            }
        }
    }
}

pub fn read_operand(
    bus: &SharedBus,
    registers: &Registers,
    op: &Operand) -> Result<i64> {
    match op {
        Operand::Reg(reg) => Ok(registers.get(*reg) as i64),
        Operand::Imm(val) => Ok(*val),
        Operand::Memory { base, index, disp, size, segment: _ } => {
            let addr = calc_address(registers, *base, *index, *disp);
            let val = match size {
                OperandSize::Byte => {
                    let byte = bus.borrow().read_u8(addr)?;
                    byte as i64
                },
                OperandSize::Word => {
                    let word = bus.borrow().read_u16(addr)?;
                    word as i64
                },
                OperandSize::Dword => {
                    let dword = bus.borrow().read_u32(addr)?;
                    dword as i64
                },
                OperandSize::Qword => {
                    let qword = bus.borrow().read_u64(addr)?;
                    qword as i64
                },
            };
            Ok(val)
        }
    }
}

pub fn write_operand_u64(
    bus: &SharedBus,
    registers: &mut Registers,
    operand: Operand,
    value: u64
) -> Result<()> {
    match operand {
        Operand::Reg(reg) => {
            registers.set(reg, value);
        },
        Operand::Memory { base, index, disp, size, segment } => {
            let addr = calc_address(&registers, base, index, disp);
            bus.borrow_mut().write_u64(addr, value)?;
        },
        Operand::Imm(_) => {
            bail!("Invalid pop operand: immediate");
        },
    }

    Ok(())
}

pub fn write_operand(
    bus: &SharedBus,
    registers: &mut Registers,
    op: Operand,
    value: u64) -> Result<()> {
    match op {
        Operand::Reg(reg) => {
            registers.set(reg, value);
            Ok(())
        }
        Operand::Memory { base, index, disp, size, segment: _ } => {
            let addr = calc_address(registers, base, index, disp);
            match size {
                OperandSize::Byte => bus.borrow_mut().write_u8(addr, value as u8),
                OperandSize::Word => {
                    let val = value as u16;
                    let bytes = val.to_le_bytes();
                    bus.borrow_mut().write_bytes(addr, &bytes)
                },
                OperandSize::Dword => {
                    let val = value as u32;
                    let bytes = val.to_le_bytes();
                    bus.borrow_mut().write_bytes(addr, &bytes)
                },
                OperandSize::Qword => bus.borrow_mut().write_u64(addr, value),
            }
        }
        Operand::Imm(_) => {
            Err(anyhow::anyhow!("Cannot write to immediate operand"))
        }
    }
}