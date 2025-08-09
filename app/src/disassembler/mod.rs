use std::{fs::File, io::{BufWriter, Read, Write}, path::Path};
use anyhow::Result;
use capstone::{
    arch::{
        self, x86::{X86OperandIterator, X86OperandType, X86Reg}, BuildsCapstone, BuildsCapstoneSyntax, DetailsArchInsn
    }, Capstone, InsnDetail, InsnGroupId, InsnGroupType::{CS_GRP_CALL, CS_GRP_JUMP}, InsnId, RegId
};
use capstone::arch::x86::X86Insn::{X86_INS_INT3, X86_INS_NOP};
use log::*;
use serde::de;

use crate::disassembler::stream::DisasmStream;

mod types;
mod stream;

pub struct Disassembler {
    cs: Capstone,
}

impl Disassembler {
    pub fn new() -> Result<Self> {
        let mut cs = Capstone::new()
            .x86()
            .mode(arch::x86::ArchMode::Mode64)
            .syntax(arch::x86::ArchSyntax::Intel)
            .build()?;

        cs.set_skipdata(true)?;
        cs.set_detail(true)?;

        Ok(Self { cs })
    }

    pub fn export_to_txt(&self, base_addr: u64, reader: impl Read, file_path: &Path) -> Result<()> {

        let mut file = File::create(file_path)?;
        let mut writer = BufWriter::new(file);

        let buf_size = 10000;
        let mut stream = DisasmStream::new(reader, buf_size)?;

        while let Ok(batch) = stream.next_batch() {

        }

        Ok(())
    }

    pub fn export_to_csv(&self, base_addr: u64, data: &[u8], file_path: &Path) -> Result<()> {
        let mut file = File::create(file_path)?;
        let mut writer = BufWriter::new(file);
        self.write_csv_header(&mut writer)?;

        let chunk_size = 10000;
        let mut offset = 0;
        let mut address = base_addr;
        let mut nop_group = (0, 0);
        let mut int3_group: Option<(u64, u64)> = None;

        loop {
            if offset >= data.len() {
                break;
            }

            let bytes = &data[offset..];

            let instructions = self.cs.disasm_count(&data, address, chunk_size)?;

            let first_address = &instructions.first().unwrap().address();
            let end_address = &instructions.last().unwrap().address();
            info!("0x{:X} 0x{:X}", first_address, end_address);
            
            for instruction in instructions.iter() {
                address = instruction.address();
                offset += instruction.len();
                let mnemonic = instruction.mnemonic().unwrap_or("");
                let op_str = instruction.op_str().unwrap_or("");
                let instr_size = instruction.len() as u64;
                let id = instruction.id();

                if id == InsnId(X86_INS_INT3 as u32){
                    int3_group = Some(match int3_group {
                        Some((start, _)) => (start, address),
                        None => (address, address),
                    });
                    continue;
                } else if let Some((start, end)) = int3_group.take() {
                    self.write_grouped_instruction(&mut writer, start, end, "int3")?;
                }

                if id == InsnId(X86_INS_NOP as u32) {
                    if nop_group.0 == 0 {
                        nop_group.1 = address;
                    }
                    nop_group.0 += 1;
                    continue;
                } else if nop_group.0 > 0 {
                    let start_addr = nop_group.1;
                    let end_addr = start_addr + nop_group.0 as u64;
                    self.write_grouped_instruction(&mut writer, start_addr, end_addr, "nop")?;
                    nop_group = (0, 0);
                }

                let detail = self.cs.insn_detail(instruction)?;
                let arch_detail = detail.arch_detail();
                let x86_detail = arch_detail.x86().unwrap();

                let (imm_val, imm_addr, rip_target, displacement) =
                    Self::extract_operand_info(mnemonic, x86_detail.operands(), address, instr_size, &detail);

                self.write_instruction_csv(
                    &mut writer,
                    instruction.address(),
                    mnemonic,
                    op_str,
                    &imm_val,
                    &imm_addr,
                    &rip_target,
                    &displacement,
                )?;
            }
        }

        // Final grouped outputs
        if let Some((start, end)) = int3_group.take() {
            self.write_grouped_instruction(&mut writer, start, end, "int3")?;
        }
        if nop_group.0 > 0 {
            let start = nop_group.1;
            let end = start + nop_group.0 as u64;
            self.write_grouped_instruction(&mut writer, start, end, "nop")?;
        }

        Ok(())
    }

    fn write_csv_header<W: Write>(&self, writer: &mut W) -> Result<()> {
        writeln!(
            writer,
            "address,mnemonic;op_str;immediate_value;immediate_address;rip_target;displacement;end_addr_range"
        )?;
        Ok(())
    }

    fn write_grouped_instruction<W: Write>(
        &self,
        writer: &mut W,
        start: u64,
        end: u64,
        label: &str,
    ) -> Result<()> {
        writeln!(
            writer,
            "0x{:X};{};;;;;;;0x{:X}",
            start, label, end
        )?;
        Ok(())
    }

    fn write_instruction_csv<W: Write>(
        &self,
        writer: &mut W,
        address: u64,
        mnemonic: &str,
        op_str: &str,
        imm_val: &str,
        imm_addr: &str,
        rip_target: &str,
        displacement: &str,
    ) -> Result<()> {
        writeln!(
            writer,
            "0x{:X};{};{};{};{};{};{};",
            address,
            mnemonic,
            op_str,
            imm_val,
            imm_addr,
            rip_target,
            displacement
        )?;
        Ok(())
    }

    fn extract_operand_info(
        mnemonic: &str,
        operands: X86OperandIterator,
        instr_addr: u64,
        instr_size: u64,
        detail: &InsnDetail
    ) -> (String, String, String, String) {
        let mut imm_val = String::new();
        let mut imm_addr = String::new();
        let mut rip_target = String::new();
        let mut displacement = String::new();

        let suppress_imm_val = detail
            .groups()
            .iter()
            .any(|&group| group == InsnGroupId(CS_GRP_JUMP as u8) 
                || group == InsnGroupId(CS_GRP_CALL as u8));

        for op in operands {
            match op.op_type {
                X86OperandType::Imm(imm) => {
                    if suppress_imm_val {          
                        imm_addr = format!("0x{:X}", imm);
                    }
                    else {
                        imm_val = format!("0x{:X}", imm);
                    }
                }
                X86OperandType::Mem(mem) => {

                    let disp = mem.disp();
                    displacement = disp.to_string();
                    if mem.base() == RegId(X86Reg::X86_REG_RIP as u16) {
                        let target = instr_addr as i64 + instr_size as i64 + disp;
                        rip_target = format!("0x{:X}", target);
                    }
                }
                _ => {}
            }
        }

        (imm_val, imm_addr, rip_target, displacement)
    }
}
