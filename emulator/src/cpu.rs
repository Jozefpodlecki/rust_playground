use decompiler_lib::decompiler::types::{ConditionCode, Instruction, InstructionType, Operand, OperandSize, Register};
use anyhow::{bail, Result};
use log::info;
use crate::{bus::SharedBus, registers::Registers};

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

        match instruction.kind {
            InstructionType::Invalid => bail!("Invalid opcode"),
            InstructionType::Shr(dst, count_operand) => {
                let value = self.read_operand(&dst)? as u64;

                let count = match count_operand {
                    Operand::Imm(c) => c as u8,
                    Operand::Reg(reg) => (self.registers.get(reg) & 0xFF) as u8, 
                    _ => bail!("Invalid shift count operand"),
                };

                info!("{} {} {}", value, count, value >> count);
                let result = value >> count;
                self.write_operand(dst, result)?;

                // Update flags
                // Carry Flag = last bit shifted out (bit count-1 of original val)
                if count <= 64 {
                    let carry_bit = (value >> (count - 1)) & 1;
                    if carry_bit == 1 {
                        self.rflags |= 1 << 0; // CF
                    } else {
                        self.rflags &= !(1 << 0);
                    }
                }

                // Overflow Flag is defined only if count == 1
                if count == 1 {
                    // OF = most significant bit before shift
                    let msb = (value >> 63) & 1;
                    if msb == 1 {
                        self.rflags |= 1 << 11;
                    } else {
                        self.rflags &= !(1 << 11);
                    }
                } else {
                    // Otherwise, clear OF
                    self.rflags &= !(1 << 11);
                }

                // Update Zero Flag (ZF)
                if result == 0 {
                    self.rflags |= 1 << 6;
                } else {
                    self.rflags &= !(1 << 6);
                }

                // Update Sign Flag (SF) - Most significant bit of result
                if (result >> 63) & 1 == 1 {
                    self.rflags |= 1 << 7;
                } else {
                    self.rflags &= !(1 << 7);
                }

                // Parity Flag (PF) - set if lowest byte of result has even parity
                let low_byte = (result & 0xFF) as u8;
                self.update_parity_flag(low_byte);
            },
            InstructionType::Pop(operand) => {
                let value = self.bus.borrow().read_u64(self.registers.rsp)?;

                self.registers.rsp = self.registers.rsp.wrapping_add(8);

                match operand {
                    Operand::Reg(reg) => {
                        self.registers.set(reg, value);
                    },
                    Operand::Memory { base, index, disp, size, segment } => {
                        let addr = self.calc_address(base, index, disp);
                        self.bus.borrow_mut().write_u64(addr, value)?;
                    },
                    Operand::Imm(_) => {
                        bail!("Invalid pop operand: immediate");
                    },
                }
            }
            InstructionType::Mov(src, dst) => {
                let value = self.read_operand(&src)?;
                self.write_operand(dst, value as u64)?;
            }
            InstructionType::Xor(op1, op2) => {
                let val1 = self.read_operand(&op1)? as u64;
                let val2 = self.read_operand(&op2)? as u64;
                let result = val1 ^ val2;

                self.write_operand(op1, result)?;

                // Clear CF and OF
                self.rflags &= !((1 << 0) | (1 << 11));

                // Set ZF
                if result == 0 {
                    self.rflags |= 1 << 6;
                } else {
                    self.rflags &= !(1 << 6);
                }

                // Set SF (sign flag, highest bit for 64-bit)
                if (result >> 63) & 1 == 1 {
                    self.rflags |= 1 << 7;
                } else {
                    self.rflags &= !(1 << 7);
                }

                if result.count_ones() % 2 == 0 {
                    self.rflags |= 1 << 2;
                } else {
                    self.rflags &= !(1 << 2);
                }
            }
            InstructionType::Test(op1, op2) => {
                let val1 = self.read_operand(&op1)?;
                let val2 = self.read_operand(&op2)?;
                let result = val1 & val2;
                self.update_flags_logical(result as u64);
            }
            InstructionType::Leave => {
                self.registers.rsp = self.registers.rbp;
                let new_rbp = self.bus.borrow().read_u64(self.registers.rsp)?;
                self.registers.rbp = new_rbp;
                self.registers.rsp = self.registers.rsp.wrapping_add(8);
            }
            InstructionType::Cmp(op1, op2) => {
                let val1 = self.read_operand(&op1)?;
                let val2 = self.read_operand(&op2)?;
                let result = val1.wrapping_sub(val2);
                self.update_flags_sub(val1 as u64, val2 as u64, result as u64);
            }
            InstructionType::Inc(op) => {
                let value = self.read_operand(&op)?;
                let result = value.wrapping_add(1);
                info!("{} {}", value, result);
                self.write_operand(op, result as u64)?;
                self.update_flags_add(value as u64, 1, result as u64);
            }
            InstructionType::Dec(op) => {
                let value = self.read_operand(&op)?;
                let result = value.wrapping_sub(1);
                info!("{} {}", value, result);
                self.write_operand(op, result as u64)?;
                self.update_flags_sub_no_cf(value as u64, 1, result as u64);
            }
            InstructionType::Add(op1, op2) => {
                let val1 = self.read_operand(&op1)?;
                let val2 = self.read_operand(&op2)?;
                let result = val1.wrapping_add(val2);
                self.write_operand(op1, result as u64)?;
                self.update_flags_add(val1 as u64, val2 as u64, result as u64);
            }
            InstructionType::Sub(op1, op2) => {
                let val1 = self.read_operand(&op1)?;
                let val2 = self.read_operand(&op2)?;
                let result = val1.wrapping_sub(val2);
                self.write_operand(op1, result as u64)?;
                self.update_flags_sub(val1 as u64, val2 as u64, result as u64);
            }
            InstructionType::Push(operand) => {
                self.registers.rsp = self.registers.rsp.wrapping_sub(8);
                let value = self.read_operand(&operand)? as u64;
                self.bus.borrow_mut().write_u64(self.registers.rsp, value)?;
            },
            InstructionType::ConditionalJump(cond, target) => {
                if self.evaluate_condition(cond)? {
                    self.rip = target;
                }
            },
            InstructionType::UnconditionalJump(target) => {
                let target_addr = self.read_operand(&target)? as u64;
                self.rip = target_addr;
            },
            InstructionType::Call(operand) => {
                let return_addr = self.rip + instruction.length;
                self.registers.rsp = self.registers.rsp.wrapping_sub(8);
                self.bus.borrow_mut().write_u64(self.registers.rsp, return_addr)?;

                match operand {
                    Operand::Imm(addr) => {
                        self.rip = addr as u64;
                    },
                    Operand::Reg(reg) => {
                        let addr = self.registers.get(reg);
                        self.rip = addr;
                    },
                    Operand::Memory { base, index, disp, size, segment } => {
                        let base_val = base.map_or(0, |r| self.registers.get(r));
                        let index_val = index.map_or(0, |(r, scale)| self.registers.get(r) * scale as u64);
                        let addr = base_val.wrapping_add(index_val).wrapping_add(disp as u64);
                        self.rip = addr;
                    },
                }
            },
            InstructionType::Ret => {
                let return_addr = self.bus.borrow().read_u64(self.registers.rsp)?;
                self.registers.rsp = self.registers.rsp.wrapping_add(8);
                self.rip = return_addr;
            },
            _ => {}
        }

        if self.rip == old_rip {
            self.rip = self.rip.wrapping_add(instruction.length);
        }

        Ok(())
    }

    fn update_parity_flag(&mut self, byte: u8) {
        // count number of set bits in byte
        let mut count = 0;
        let mut val = byte;
        for _ in 0..8 {
            count += val & 1;
            val >>= 1;
        }
        if count % 2 == 0 {
            self.rflags |= 1 << 2;  // PF
        } else {
            self.rflags &= !(1 << 2);
        }
    }

    pub fn evaluate_condition(&self, cond: ConditionCode) -> Result<bool> {
        let rflags = self.rflags;
        let cf = (rflags >> 0) & 1 != 0;  // Carry Flag
        let pf = (rflags >> 2) & 1 != 0;  // Parity Flag
        let af = (rflags >> 4) & 1 != 0;  // Adjust Flag (optional)
        let zf = (rflags >> 6) & 1 != 0;  // Zero Flag
        let sf = (rflags >> 7) & 1 != 0;  // Sign Flag
        let of = (rflags >> 11) & 1 != 0; // Overflow Flag

        Ok(match cond {
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
            ConditionCode::CXZ => self.registers.rcx == 0, // RCX == 0
        })
    }

    fn read_operand(&self, op: &Operand) -> Result<i64> {
        match op {
            Operand::Reg(reg) => Ok(self.registers.get(*reg) as i64),
            Operand::Imm(val) => Ok(*val),
            Operand::Memory { base, index, disp, size, segment: _ } => {
                let addr = self.calc_address(*base, *index, *disp);
                let val = match size {
                    OperandSize::Byte => {
                        let byte = self.bus.borrow().read_u8(addr)?;
                        byte as i64
                    },
                    OperandSize::Word => {
                        let word = self.bus.borrow().read_u16(addr)?;
                        word as i64
                    },
                    OperandSize::Dword => {
                        let dword = self.bus.borrow().read_u32(addr)?;
                        dword as i64
                    },
                    OperandSize::Qword => {
                        let qword = self.bus.borrow().read_u64(addr)?;
                        qword as i64
                    },
                };
                Ok(val)
            }
        }
    }

    fn update_flags_logical(&mut self, result: u64) {
        // Clear CF and OF flags (bits 0 and 11)
        self.rflags &= !((1 << 0) | (1 << 11));

        // Set or clear Zero Flag (ZF, bit 6)
        if result == 0 {
            self.rflags |= 1 << 6;
        } else {
            self.rflags &= !(1 << 6);
        }

        // Set or clear Sign Flag (SF, bit 7)
        if (result >> 63) & 1 == 1 {
            self.rflags |= 1 << 7;
        } else {
            self.rflags &= !(1 << 7);
        }
    }

    fn calc_address(
            &self,
            base: Option<Register>,
            index: Option<(Register, u8)>,
            disp: i64
        ) -> u64 {
            let base_val = base.map_or(0, |r| self.registers.get(r));
            let index_val = index.map_or(0, |(r, scale)| self.registers.get(r) * scale as u64);
            base_val.wrapping_add(index_val).wrapping_add(disp as u64)
        }


    fn write_operand(&mut self, op: Operand, value: u64) -> Result<()> {
        match op {
            Operand::Reg(reg) => {
                self.registers.set(reg, value);
                Ok(())
            }
            Operand::Memory { base, index, disp, size, segment: _ } => {
                let addr = self.calc_address(base, index, disp);
                match size {
                    OperandSize::Byte => self.bus.borrow_mut().write_u8(addr, value as u8),
                    OperandSize::Word => {
                        let val = value as u16;
                        let bytes = val.to_le_bytes();
                        self.bus.borrow_mut().write_bytes(addr, &bytes)
                    },
                    OperandSize::Dword => {
                        let val = value as u32;
                        let bytes = val.to_le_bytes();
                        self.bus.borrow_mut().write_bytes(addr, &bytes)
                    },
                    OperandSize::Qword => self.bus.borrow_mut().write_u64(addr, value),
                }
            }
            Operand::Imm(_) => {
                Err(anyhow::anyhow!("Cannot write to immediate operand"))
            }
        }
    }

    fn update_flags_add(&mut self, op1: u64, op2: u64, result: u64) {
        let rflags = &mut self.rflags;

        // Zero Flag (ZF): Set if result == 0
        if result == 0 {
            *rflags |= 1 << 6;
        } else {
            *rflags &= !(1 << 6);
        }

        // Sign Flag (SF): Set if most significant bit is set (for 64-bit, bit 63)
        if (result >> 63) & 1 == 1 {
            *rflags |= 1 << 7;
        } else {
            *rflags &= !(1 << 7);
        }

        // Carry Flag (CF): Set if unsigned overflow occurs
        if result < op1 {
            *rflags |= 1 << 0;
        } else {
            *rflags &= !(1 << 0);
        }

        // Overflow Flag (OF): Set if signed overflow occurs
        let op1_sign = (op1 >> 63) & 1;
        let op2_sign = (op2 >> 63) & 1;
        let res_sign = (result >> 63) & 1;
        if op1_sign == op2_sign && op1_sign != res_sign {
            *rflags |= 1 << 11;
        } else {
            *rflags &= !(1 << 11);
        }

        // You can add Parity Flag (PF) and others similarly
    }

    fn update_flags_sub_no_cf(&mut self, lhs: u64, rhs: u64, result: u64) {
        // CF is not modified
        // ZF
        if result == 0 {
            self.rflags |= 1 << 6;
        } else {
            self.rflags &= !(1 << 6);
        }
        // SF
        if (result >> 63) & 1 == 1 {
            self.rflags |= 1 << 7;
        } else {
            self.rflags &= !(1 << 7);
        }
        // OF
        let lhs_s = lhs as i64;
        let rhs_s = rhs as i64;
        let res_s = result as i64;
        if (lhs_s < 0 && rhs_s > 0 && res_s >= 0) ||
        (lhs_s >= 0 && rhs_s < 0 && res_s < 0) {
            self.rflags |= 1 << 11;
        } else {
            self.rflags &= !(1 << 11);
        }
        // PF
        if (result & 0xFF).count_ones() % 2 == 0 {
            self.rflags |= 1 << 2;
        } else {
            self.rflags &= !(1 << 2);
        }
    }
    
    fn update_flags_sub(&mut self, op1: u64, op2: u64, result: u64) {
        let rflags = &mut self.rflags;

        // Zero Flag (ZF)
        if result == 0 {
            *rflags |= 1 << 6;
        } else {
            *rflags &= !(1 << 6);
        }

        // Sign Flag (SF)
        if (result >> 63) & 1 == 1 {
            *rflags |= 1 << 7;
        } else {
            *rflags &= !(1 << 7);
        }

        // Carry Flag (CF): Set if borrow (if op1 < op2)
        if op1 < op2 {
            *rflags |= 1 << 0;
        } else {
            *rflags &= !(1 << 0);
        }

        // Overflow Flag (OF): Signed overflow detection for subtraction
        let op1_sign = (op1 >> 63) & 1;
        let op2_sign = (op2 >> 63) & 1;
        let res_sign = (result >> 63) & 1;
        if op1_sign != op2_sign && op1_sign != res_sign {
            *rflags |= 1 << 11;
        } else {
            *rflags &= !(1 << 11);
        }
    }
}
