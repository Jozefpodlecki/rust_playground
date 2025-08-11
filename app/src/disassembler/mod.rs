use std::{fs::File, io::{BufWriter, Read, Write}, path::Path};
use anyhow::Result;
use capstone::{
    arch::{
        self, x86::{X86OperandIterator, X86OperandType, X86Reg}, BuildsCapstone, BuildsCapstoneSyntax, DetailsArchInsn
    }, Capstone, InsnDetail, InsnGroupId, InsnGroupType::{CS_GRP_CALL, CS_GRP_JUMP}, InsnId, RegId
};
use log::*;
use serde::de;

use crate::disassembler::stream::DisasmStream;

mod types;
pub mod stream;

pub struct Disassembler;

impl Disassembler {
    pub fn new() -> Result<Self> {

        Ok(Self { })
    }

    pub fn export_to_txt(&self, base_addr: u64, reader: impl Read, file_path: &Path) -> Result<()> {

        let mut file = File::create(file_path)?;
        let mut writer = BufWriter::new(file);

        let buf_size = 10000;
        let mut stream = DisasmStream::new(reader, base_addr, buf_size)?;

        for instr in stream {
            writeln!(writer, "{}", instr)?;
        }

        Ok(())
    }

    pub fn export_to_csv(&self, base_addr: u64, data: &[u8], file_path: &Path) -> Result<()> {
        let mut file = File::create(file_path)?;
        let mut writer = BufWriter::new(file);
        self.write_csv_header(&mut writer)?;

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
