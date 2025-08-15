use std::{collections::HashMap, fs::File, io::{BufWriter, Read, Write}, path::Path};
use anyhow::Result;

use crate::disassembler::{stream::DisasmStream, types::{InstructionType, Operand, Register}};

pub trait DisassemblerExtensions {
    fn get_calls(self) -> Result<HashMap<String, String>>;
    fn export_to_txt(self, file_path: &Path) -> Result<()>;
}

impl<R: Read> DisassemblerExtensions for DisasmStream<R> {
    fn export_to_txt(self, file_path: &Path) -> Result<()> {
        let file = File::create(file_path)?;
        let mut writer = BufWriter::new(file);

        struct PaddingGroup {
            start_addr: u64,
            end_addr: u64,
            count: u32,
        }

        let mut int3_padding_group: Option<PaddingGroup> = None;

        for instr in self {

            if instr.kind == InstructionType::Nop
             || instr.kind == InstructionType::Invalid {
                continue;
            }

            if instr.kind == InstructionType::Int3 {
                if let Some(group) = int3_padding_group.as_mut() {
                    group.end_addr = instr.address;
                    group.count += 1;
                }
                else {
                    int3_padding_group = Some(PaddingGroup {
                        start_addr: instr.address,
                        end_addr: instr.address,
                        count: 1
                    })
                }
            }
            else {
                if let Some(group) = int3_padding_group.as_mut() {
                    if group.count == 1 {
                        writeln!(writer, "{}", instr)?;
                        int3_padding_group = None;
                    }
                    else {
                        writeln!(writer, "0xx{:X} - 0xx{:X} int3", group.start_addr, group.end_addr)?;
                        int3_padding_group = None;
                    }
                }

                writeln!(writer, "{}", instr)?;
            }
        }

        Ok(())
    }
    
    fn get_calls(self) -> Result<HashMap<String, String>> {
        let mut map: HashMap<String, String> = HashMap::new();

        for instr in self {
            match instr.kind {
                InstructionType::Call(operand) => {
                    match operand {
                        Operand::Reg(reg) => {
                            let key = format!("0x{:X}", instr.address);
                            let value = format!("{:?}", reg);
                            map.insert(key, value);
                        },
                        Operand::Imm(value) => {
                            let key = format!("0x{:X}", instr.address);
                            let value = format!("0x{:X}", value);
                            map.insert(key, value);
                        },
                        Operand::Memory { base, disp, .. } => {
                            let key = format!("0x{:X}", instr.address);
                            if let Some(Register::RIP) = base {
                                let rip_target = instr.address.wrapping_add(instr.length).wrapping_add(disp as u64);
                                let value = format!("0x{:X}", rip_target);
                                map.insert(key, value);
                            }
                            else {
                                let value = instr.op_str;
                                map.insert(key, value);
                            }
                        },
                    }
                },
                _ => continue
            }
        }

        Ok(map)
    }
}