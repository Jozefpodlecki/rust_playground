use std::{collections::HashMap, fs::File, io::Read, path::{Path, PathBuf}};

use decompiler_lib::decompiler::types::{ConditionCode, Operand, OperandSize, Register};
use anyhow::{bail, Result};
use log::{debug, info};
use serde_json::Value;
use crate::{bus::SharedBus, emulator::{snapshot::Snapshot, Bus, MemoryRegion}, registers::Registers};

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
    bus: &Bus,
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
            debug!("read_operand_u64_rip 0x{:X}", addr);
            addr
        },
    }
}

pub fn read_operand_u64(
    bus: &Bus,
    registers: &Registers,
    op: Operand
) -> Result<u64> {
    Ok(match op {
        Operand::Reg(reg) => registers.get(reg),
        Operand::Imm(val) => val as u64,
        Operand::Memory { base, index, disp, size, segment: _ } => {
            let addr = calc_address(&registers, base, index, disp);
            match size {
                OperandSize::Byte => bus.read_u8(addr)? as u64,
                OperandSize::Word => bus.read_u16(addr)? as u64,
                OperandSize::Dword => bus.read_u32(addr)? as u64,
                OperandSize::Qword => bus.read_u64(addr)?,
            }
        }
    })
}

pub fn read_operand(
    bus: &Bus,
    registers: &Registers,
    op: &Operand) -> Result<i64> {
    match op {
        Operand::Reg(reg) => Ok(registers.get(*reg) as i64),
        Operand::Imm(val) => Ok(*val),
        Operand::Memory { base, index, disp, size, segment: _ } => {
            let addr = calc_address(registers, *base, *index, *disp);
            let val = match size {
                OperandSize::Byte => bus.read_u8(addr)? as i64,
                OperandSize::Word => bus.read_u16(addr)? as i64,
                OperandSize::Dword => bus.read_u32(addr)? as i64,
                OperandSize::Qword => bus.read_u64(addr)? as i64
            };
            Ok(val)
        }
    }
}

pub fn write_operand_u64(
    bus: &mut Bus,
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
            debug!("write_operand_u64 0x{:X} -> 0x{:X}", addr, value);
            bus.write_u64(addr, value)?;
        },
        Operand::Imm(_) => {
            bail!("Invalid pop operand: immediate");
        },
    }

    Ok(())
}

pub fn write_operand(
    bus: &mut Bus,
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
                OperandSize::Byte => {
                    debug!("write_operand 0x{:X} -> 0x{:X}", addr, value);
                    bus.write_u8(addr, value as u8)
                },
                OperandSize::Word => {
                    let val = value as u16;
                    let bytes = val.to_le_bytes();
                    debug!("write_operand 0x{:X} -> 0x{:X}", addr, value);
                    bus.write_bytes(addr, &bytes)
                },
                OperandSize::Dword => {
                    let val = value as u32;
                    let bytes = val.to_le_bytes();
                    debug!("write_operand 0x{:X} -> 0x{:X}", addr, value);
                    bus.write_bytes(addr, &bytes)
                },
                OperandSize::Qword => {
                    debug!("write_operand 0x{:X} -> 0x{:X}", addr, value);
                    bus.write_u64(addr, value)
                }
            }
        }
        Operand::Imm(_) => {
            Err(anyhow::anyhow!("Cannot write to immediate operand"))
        }
    }
}

pub fn get_memory_region(file_path: &Path) -> Result<MemoryRegion> {
    let (start_addr, size) = {
        let stem = file_path.file_stem().unwrap().to_string_lossy();
        let mut parts = stem.split('_');
        let addr_str = parts.next().unwrap();
        let size_str = parts.next().unwrap();
        
        let addr_val = u64::from_str_radix(addr_str.trim_start_matches("0x"), 16).unwrap();
        let size_val = size_str.parse::<usize>().unwrap();

        (addr_val, size_val)
    };

    let mut file = File::open(file_path)?;
    let mut data = vec![];
    file.read_to_end(&mut data)?;

    let mut region = MemoryRegion::new(start_addr, size);
    region.write_bytes(start_addr, &data)?;

    Ok(region)
}

pub fn create_stack() -> MemoryRegion {
    let stack_size = 64 * 1024usize;
    let stack_base = 0x7fff_ffff_0000 as u64;
    MemoryRegion::new(stack_base, stack_size)
}

pub fn create_snapshot() -> Result<Snapshot> {
    
    let mut regions = vec![];
    let base_path = PathBuf::from(r"C:\repos\rust_playground\app\target\debug\output\LOSTARK\PE\");

    let file_path = base_path.join(r"summary.json");
    let file = File::open(file_path)?;
    let map: HashMap<String, Value> = serde_json::from_reader(file)?;
    let value = map.get("entry_point_va").unwrap();
    let rip = u64::from_str_radix(value.as_str().unwrap().trim_start_matches("0x"), 16).unwrap();

    let file_path = base_path.join(r"0x147E25000_4096_bpcbpmed.section");
    let region = get_memory_region(&file_path)?;
    regions.push(region);

    let file_path = base_path.join(r"0x1475C3000_8790016_nuztkydr.section");
    let region = get_memory_region(&file_path)?;
    regions.push(region);

    let file_path = base_path.join(r"0x140000000_4096_dos.data");
    let region = get_memory_region(&file_path)?;
    regions.push(region);

    let file_path = base_path.join(r"0x1469EA000_12423168_2020202020202020.section");
    let region = get_memory_region(&file_path)?;
    regions.push(region);

    let region = create_stack();
    let rsp = region.end_addr;
    regions.push(region);

    Ok(Snapshot { 
        rip,
        rflags: 0,
        registers: Registers::new(rsp),
        regions
    })
}