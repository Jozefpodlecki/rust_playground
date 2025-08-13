use decompiler_lib::decompiler::types::{Instruction, InstructionType, Operand, OperandSize, RepeatableInstruction};
use anyhow::{bail, Result};
use log::info;
use crate::{alu, bus::{Bus, SharedBus}, flags::*, registers::Registers, utils::*};

#[derive(Debug, Default, Clone)]
pub struct Cpu {
    pub registers: Registers,
    pub rip: u64,
    pub rflags: u64,
    pub bus: SharedBus,
}


impl Cpu {
    pub fn new(
        rip: u64,
        rsp: u64,
        bus: SharedBus) -> Self {
        Self {
            registers: Registers::new(rsp),
            rip,
            rflags: 0,
            bus
        }
    }

    pub fn handle(&mut self, instruction: Instruction) -> Result<()> {
        let old_rip = self.rip;

        // if let InstructionType::Rep(_) = instruction.kind {
        //     info!("{:?}", instruction);
        //     bail!("test");
        // }

        match instruction.kind {
            InstructionType::Invalid => bail!("Invalid opcode"),
            InstructionType::Shl(dst, count_operand) => {
                let val = read_operand(&self.bus, &self.registers, &dst)? as u64;
                let count = get_count(&self.registers, count_operand)?;
                let result = alu::shl(&mut self.rflags, val, count);
                write_operand(&self.bus, &mut self.registers, dst, result)?;
            }
            InstructionType::Shr(dst, count_operand) => {
                let val = read_operand(&self.bus, &self.registers, &dst)? as u64;
                let count = get_count(&self.registers, count_operand)?;
                let result = alu::shr(&mut self.rflags, val, count);
                write_operand(&self.bus, &mut self.registers, dst, result)?;
            },
            InstructionType::Pop(dst) => {
                let value = self.bus.borrow().read_u64(self.registers.rsp)?;
                self.registers.rsp = self.registers.rsp.wrapping_add(8);
                write_operand_u64(&self.bus, &mut self.registers, dst, value)?;
            }
            InstructionType::MovZX(src, dst) => {
                let src_val = read_operand_u64(&self.bus, &mut self.registers, src);
                write_operand(&self.bus, &mut self.registers, dst, src_val)?;
            }
            InstructionType::Mov(src, dst) => {
                let value = read_operand(&self.bus, &self.registers,  &src)?;
                write_operand(&self.bus, &mut self.registers, dst, value as u64)?;
            }
            InstructionType::Xor(op1, op2) => {
                let val1 = read_operand(&self.bus, &self.registers, &op1)? as u64;
                let val2 = read_operand(&self.bus, &self.registers, &op2)? as u64;
                let result = val1 ^ val2;

                write_operand(&self.bus, &mut self.registers, op1, result)?;
                update_flags_after_logic(&mut self.rflags, result);
            }
            InstructionType::Adc(op1, op2) => {
                let val1 = read_operand(&self.bus, &self.registers, &op1)? as u64;
                let val2 = read_operand(&self.bus, &self.registers, &op2)? as u64;
                let carry = (self.rflags & 1) as u64;

                let intermediate = val1.wrapping_add(val2);
                let result = intermediate.wrapping_add(carry);

                write_operand(&self.bus, &mut self.registers,op1, result)?;
                update_flags_adc(&mut self.rflags, val1, val2, carry, result);
            }
            InstructionType::Test(op1, op2) => {
                let val1 = read_operand(&self.bus, &self.registers, &op1)?;
                let val2 = read_operand(&self.bus, &self.registers, &op2)?;
                let result = val1 & val2;
                update_flags_logical(&mut self.rflags, result as u64);
            }
            InstructionType::Leave => {
                self.registers.rsp = self.registers.rbp;
                let new_rbp = self.bus.borrow().read_u64(self.registers.rsp)?;
                self.registers.rbp = new_rbp;
                self.registers.rsp = self.registers.rsp.wrapping_add(8);
            }
            InstructionType::Cmp(op1, op2) => {
                let val1 = read_operand(&self.bus, &self.registers, &op1)?;
                let val2 = read_operand(&self.bus, &self.registers, &op2)?;
                let result = val1.wrapping_sub(val2);
                // info!("0x{:X} 0x{:X} {}", val1, val2, result);
                update_flags_sub(&mut self.rflags, val1 as u64, val2 as u64, result as u64);
            }
            InstructionType::Inc(op) => {
                let value = read_operand(&self.bus, &self.registers, &op)?;
                let result = value.wrapping_add(1);
                // info!("{} {}", value, result);
                write_operand(&mut self.bus, &mut self.registers, op, result as u64)?;
                update_flags_add(&mut self.rflags, value as u64, 1, result as u64);
            }
            InstructionType::Dec(op) => {
                let value = read_operand(&self.bus, &self.registers, &op)?;
                let result = value.wrapping_sub(1);
                // info!("{} {}", value, result);
                write_operand(&mut self.bus, &mut self.registers, op, result as u64)?;
                update_flags_sub_no_cf(&mut self.rflags, value as u64, 1, result as u64);
            }
            InstructionType::Add(op1, op2) => {
                let val1 = read_operand(&self.bus, &self.registers, &op1)?;
                let val2 = read_operand(&self.bus, &self.registers, &op2)?;
                let result = val1.wrapping_add(val2);
                write_operand(&mut self.bus, &mut self.registers, op1, result as u64)?;
                update_flags_add(&mut self.rflags, val1 as u64, val2 as u64, result as u64);
            }
            InstructionType::Sub(op1, op2) => {
                let val1 = read_operand(&self.bus, &self.registers, &op1)?;
                let val2 = read_operand(&self.bus, &self.registers, &op2)?;
                let result = val1.wrapping_sub(val2);
                write_operand(&mut self.bus, &mut self.registers, op1, result as u64)?;
                update_flags_sub(&mut self.rflags, val1 as u64, val2 as u64, result as u64);
            }
            InstructionType::Push(operand) => {
                self.registers.rsp = self.registers.rsp.wrapping_sub(8);
                let value = read_operand(&self.bus, &self.registers, &operand)? as u64;
                self.bus.borrow_mut().write_u64(self.registers.rsp, value)?;
            },
            InstructionType::ConditionalJump(cond, target) => {
                if evaluate_condition(self.registers.rcx, self.rflags, cond) {
                    self.rip = target;
                }
            },
            InstructionType::UnconditionalJump(target) => {
                let target_addr = read_operand(&self.bus, &self.registers, &target)? as u64;
                self.rip = target_addr;
            },
            InstructionType::Call(operand) => {
                let return_addr = self.rip + instruction.length;
                self.registers.rsp = self.registers.rsp.wrapping_sub(8);
                self.bus.borrow_mut().write_u64(self.registers.rsp, return_addr)?;

                self.rip = read_operand_u64_rip(&self.bus, &self.registers, operand);
            },
            InstructionType::Cld => {
                self.rflags &= !(1 << 10);
            },
            InstructionType::Ret => {
                let return_addr = self.bus.borrow().read_u64(self.registers.rsp)?;
                self.registers.rsp = self.registers.rsp.wrapping_add(8);
                self.rip = return_addr;
            },
            InstructionType::Rep(instr) => {
                match instr {
                    RepeatableInstruction::Mov(src, dst) => {
                        while self.registers.rcx != 0 {
                            let byte = self.bus.borrow().read_u8(self.registers.rsi)?;
                            
                            self.bus.borrow_mut().write_u8(self.registers.rdi, byte)?;

                            if (self.rflags >> 10) & 1 == 0 {
                                self.registers.rsi = self.registers.rsi.wrapping_add(1);
                                self.registers.rdi = self.registers.rdi.wrapping_add(1);
                            } else {
                                self.registers.rsi = self.registers.rsi.wrapping_sub(1);
                                self.registers.rdi = self.registers.rdi.wrapping_sub(1);
                            }

                            self.registers.rcx = self.registers.rcx.wrapping_sub(1);
                        }
                    },
                    _ => bail!("Unhandled REP instruction"),
                }
            }
            instr => bail!("Unhandled {:?}", instr)
        }

        if self.rip == old_rip {
            self.rip = self.rip.wrapping_add(instruction.length);
        }

        Ok(())
    }
}
