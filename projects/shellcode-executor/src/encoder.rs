use core::fmt::{Debug, Display, Formatter};

use heapless::Vec;
use iced_x86::{Code, Encoder, IcedError, Instruction, MemoryOperand, Register};
use toolkit::println;

#[derive(Debug)]
pub enum EncoderError {
    Encoder(iced_x86::IcedError),
    UnboundLabel
}

impl From<IcedError> for EncoderError {
    fn from(err: IcedError) -> Self {
        EncoderError::Encoder(err)
    }
}

pub struct EncoderWithRip<const N: usize> {
    encoder: Encoder,
    rip: usize,
    labels: Vec<Option<usize>, 10>,
    fixups: Vec<Fixup, 10>,
    next_label: usize,
}

#[derive(Clone, Copy)]
pub struct Label {
    id: usize,
}

struct Fixup {
    pos: usize,
    label_id: usize,
}

impl Fixup {
    fn patch(&self, buffer: &mut [u8], target: usize) {
        let disp = (target as i64 - (self.pos as i64 + 4)) as i32;
        buffer[self.pos..self.pos + 4].copy_from_slice(&disp.to_le_bytes());
    }
}

impl<const N: usize> EncoderWithRip<N> {
    pub fn new() -> Self {
        let mut encoder = Encoder::new(64);
        let mut rip = 0;

        Self {
            encoder,
            rip,
            labels: Vec::new(),
            fixups: Vec::new(),
            next_label: 0,
        }
    }

    pub fn inf_jmp_breakpoint(&mut self) -> Result<(), IcedError> {
        let loop_label = self.new_label();
        self.bind_label(loop_label);
        self.jmp(loop_label)?;
         
        Ok(())
    }

    pub fn new_label(&mut self) -> Label {
        let id = self.next_label;
        self.next_label += 1;
        self.labels.push(None);
        Label { id }
    }

    pub fn bind_label(&mut self, label: Label) {
        self.labels[label.id] = Some(self.rip);
    }

    fn emit_jump(&mut self, code: Code, label: Label) -> Result<(), IcedError> {
        let pos = self.rip;

        let instr = Instruction::with_branch(code, 0)?;
        self.encode(instr)?;

        let opcode_len = match code {
            Code::Jmp_rel32_64 => 1,
            Code::Je_rel32_64 | Code::Jne_rel32_64 => 2,
            _ => unreachable!(),
        };
        self.fixups.push(Fixup {
            pos: pos + opcode_len,
            label_id: label.id,
        });
        Ok(())
    }

    pub fn encode(&mut self, instr:Instruction) -> Result<(), IcedError> {
        let bytes = self.encoder.encode(&instr, self.rip as _)?;
        self.rip += bytes;
        Ok(())
    }

    pub fn mov_rax_gs_60(&mut self) -> Result<(), IcedError> {
        let mem = MemoryOperand::new(
            Register::None,
            Register::None,
            1,
            0x60,
            4,
            false,
            Register::GS,
        );
        self.encode(Instruction::with2(Code::Mov_r64_rm64, Register::RAX, mem)?)
    }

    pub fn mov_r64_mem(&mut self, dest: Register, base: Register, offset: i64) -> Result<(), IcedError> {
        let mem = MemoryOperand::with_base_displ(base, offset);
        self.encode(Instruction::with2(Code::Mov_r64_rm64, dest, mem)?)
    }

    pub fn int3(&mut self) -> Result<(), IcedError> {
        self.encode(Instruction::with(Code::Int3))
    }

    pub fn je(&mut self, label: Label) -> Result<(), IcedError> {
        self.emit_jump(Code::Je_rel32_64, label)
    }

    pub fn jne(&mut self, label: Label) -> Result<(), IcedError> {
        self.emit_jump(Code::Jne_rel32_64, label)
    }

    pub fn jmp(&mut self, label: Label) -> Result<(), IcedError> {
        self.emit_jump(Code::Jmp_rel32_64, label)
    }

    pub fn mov_reg_imm64(&mut self, reg: Register, val: u64) -> Result<(), IcedError> {
        self.encode(Instruction::with2(Code::Mov_r64_imm64, reg, val)?)
    }

    pub fn mov_reg_reg(&mut self, to: Register, from: Register) -> Result<(), IcedError> {
        self.encode(Instruction::with2(Code::Mov_r64_rm64, to, from)?)
    }

    pub fn mov_reg_imm32(&mut self, reg: Register, val: u32) -> Result<(), IcedError> {
        self.encode(Instruction::with2(Code::Mov_r32_imm32, reg, val as i32)?)
    }

    pub fn mov_imm(&mut self, reg: Register, val: u64) -> Result<(), IcedError> {
        self.encode(Instruction::with2(Code::Mov_r64_imm64, reg, val)?)
    }

    pub fn mov_mem_imm(&mut self, mem: MemoryOperand, val: i32) -> Result<(), IcedError> {
        self.encode(Instruction::with2(Code::Mov_rm64_imm32, mem, val)?)
    }

    pub fn lea(&mut self, reg: Register, base: Register, offset: i64) -> Result<(), IcedError> {
        let mem = MemoryOperand::with_base_displ(base, offset);
        self.encode(Instruction::with2(Code::Lea_r64_m, reg, mem)?)
    }

    pub fn lea_rsp(&mut self, reg: Register, offset: i64) -> Result<(), IcedError> {
        self.lea(reg, Register::RSP, offset)
    }

    pub fn syscall(&mut self, eax_val: u32) -> Result<(), IcedError> {
        self.encode(Instruction::with2(Code::Mov_r32_imm32, Register::EAX, eax_val as i32)?)?;
        self.encode(Instruction::with(Code::Syscall))
    }

    pub fn test_rax_rax(&mut self) -> Result<(), IcedError> {
        self.encode(Instruction::with2(Code::Test_rm64_r64, Register::RAX, Register::RAX)?)
    }

    pub fn cmp_rax_imm32(&mut self, val: u32) -> Result<(), IcedError> {
        self.encode(Instruction::with2(Code::Cmp_rm64_imm32, Register::RAX, val as i32)?)
    }

    pub fn cmp_eax_imm32(&mut self, val: u32) -> Result<(), IcedError> {
        self.encode(Instruction::with2(Code::Cmp_rm32_imm32, Register::EAX, val as i32)?)
    }

    pub fn declare_octa(&mut self, val: u128) -> Result<(), IcedError> {
        let bytes = val.to_le_bytes();
        self.encode(Instruction::with_declare_byte_16(
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
            bytes[8], bytes[9], bytes[10], bytes[11],
            bytes[12], bytes[13], bytes[14], bytes[15],
        ))
    }

    pub fn jmp_r64(&mut self, reg: Register) -> Result<(), IcedError> {
        self.encode(Instruction::with1(Code::Jmp_rm64, reg)?)
    }

    pub fn ret(&mut self) -> Result<(), IcedError> {
        self.encode(Instruction::with(Code::Retnq))
    }

    pub fn add_rsp(&mut self, val: i32) -> Result<(), IcedError> {
        self.encode(Instruction::with2(Code::Add_rm64_imm32, Register::RSP, val)?)
    }

    pub fn sub_rsp(&mut self, val: i32) -> Result<(), IcedError> {
        self.encode(Instruction::with2(Code::Sub_rm64_imm32, Register::RSP, val)?)
    }

     pub fn mov_mem_r64(&mut self, base: Register, offset: i64, reg: Register) -> Result<(), IcedError> {
        let mem = MemoryOperand::with_base_displ(base, offset);
        self.encode(Instruction::with2(Code::Mov_rm64_r64, mem, reg)?)
    }

    pub fn mov_mem_rsp_r64(&mut self, offset: i64, reg: Register) -> Result<(), IcedError> {
        self.mov_mem_r64(Register::RSP, offset, reg)
    }

    pub fn mov_mem_imm32(&mut self, base: Register, offset: i64, val: i32) -> Result<(), IcedError> {
        let mem = MemoryOperand::with_base_displ(base, offset);
        self.encode(Instruction::with2(Code::Mov_rm64_imm32, mem, val)?)
    }

    pub fn lea_rip(&mut self, reg: Register, offset: i64) -> Result<(), IcedError> {
        let mem = MemoryOperand::with_base_displ(Register::RIP, offset);
        self.encode(Instruction::with2(Code::Lea_r64_m, reg, mem)?)
    }

    pub fn lea_label(&mut self, reg: Register, label: Label) -> Result<(), IcedError> {
        let pos = self.rip;
        let mem = MemoryOperand::with_base_displ(Register::RIP, 0);
        self.encode(Instruction::with2(Code::Lea_r64_m, reg, mem)?)?;
        // LEA RIP-relative is: 48 8D 05 (3 bytes) + 4 bytes displacement = 7 bytes
        // But the actual length could vary, so we compute from pos
        let instr_len = self.rip - pos;
        self.fixups.push(Fixup {
            pos: pos + instr_len - 4,  // displacement is at the last 4 bytes
            label_id: label.id,
        });
        Ok(())
    }

    pub fn mov_mem_reg(&mut self, base: Register, offset: i64, reg: Register) -> Result<(), IcedError> {
        let mem = MemoryOperand::with_base_displ(base, offset);
        self.encode(Instruction::with2(Code::Mov_rm64_r64, mem, reg)?)
    }

    pub fn mov_mem_rsp_imm32(&mut self, offset: i64, val: i32) -> Result<(), IcedError> {
        self.mov_mem_imm32(Register::RSP, offset, val)
    }

    pub fn mov_r64_mem_rsp(&mut self, reg: Register, offset: i64) -> Result<(), IcedError> {
        let mem = MemoryOperand::with_base_displ(Register::RSP, offset);
        self.encode(Instruction::with2(Code::Mov_r64_rm64, reg, mem)?)
    }

    pub fn dec_test_r12(&mut self) -> Result<(), IcedError> {
        self.encode(Instruction::with1(Code::Dec_rm64, Register::R12)?)?;
        self.encode(Instruction::with2(Code::Test_rm64_r64, Register::R12, Register::R12)?)?;
        Ok(())
    }

    pub fn into_vec(mut self) -> Result<Vec<u8, N>, EncoderError> {
        let mut buffer = self.encoder.take_buffer();
        for fixup in &self.fixups {
            let target = self.labels[fixup.label_id].ok_or(EncoderError::UnboundLabel)?;
            fixup.patch(&mut buffer, target);
        }
        Ok(Vec::from_iter(buffer))
    }
}
