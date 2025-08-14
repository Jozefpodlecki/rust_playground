use std::u64;

use anyhow::{bail, Result};
use decompiler_lib::decompiler::types::Register;
use log::debug;
use bincode::{de::{BorrowDecoder, Decoder}, enc::Encoder, error::{DecodeError, EncodeError}, BorrowDecode, Decode, Encode};

#[repr(C)]
#[derive(Copy, Clone)]
union Reg64 {
    r64: u64,
    r32: u32x2,
    r16: u16x4,
    r8: u8x8,
}

impl Encode for Reg64 {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> std::result::Result<(), EncodeError> {
        let value = unsafe { self.r64 };
        value.encode(encoder)
    }
}

impl<Context> Decode<Context> for Reg64 {
    fn decode<D: Decoder<Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let val: u64 = Decode::decode(decoder)?;
        Ok(Self { r64: val })
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for Reg64 {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let val: u64 = Decode::decode(decoder)?;
        Ok(Self { r64: val })
    }
}

impl Default for Reg64 {
    fn default() -> Self {
        Self {
            r64: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Encode, Decode, Default, Clone)]
struct u32x2 { pub low: u32, pub high: u32 }

#[repr(C)]
#[derive(Debug, Copy, Encode, Decode, Default, Clone)]
struct u16x4 { pub w0: u16, pub w1: u16, pub w2: u16, pub w3: u16 }

#[repr(C)]
#[derive(Debug, Copy, Encode, Decode, Default, Clone)]
struct u8x8 { pub b0: u8, pub b1: u8, pub b2: u8, pub b3: u8, pub b4: u8, pub b5: u8, pub b6: u8, pub b7: u8 }


#[derive(Default, Encode, Decode, Clone)]
pub struct RegistersNew {
    pub rax: Reg64,
    pub rcx: Reg64,
    pub rdx: Reg64,
    pub rbx: Reg64,
    pub rsp: Reg64,
    pub rbp: Reg64,
    pub rsi: Reg64,
    pub rdi: Reg64,
    pub r8: Reg64,
    pub r9: Reg64,
    pub r10: Reg64,
    pub r11: Reg64,
    pub r12: Reg64,
    pub r13: Reg64,
    pub r14: Reg64,
    pub r15: Reg64,

}

#[derive(Debug, Encode, Decode, Default, Clone)]
pub struct Registers {
    pub rax: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rbx: u64,
    pub rsp: u64,
    pub rbp: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}

impl Registers {
    pub fn new(rsp: u64) -> Self {
        Self { rsp, ..Default::default() }
    }

    pub fn get(&self, reg: Register) -> u64 {
        match reg {
            Register::RAX => self.rax,
            Register::EAX => self.rax as u32 as u64,
            Register::AX  => self.rax as u16 as u64,
            Register::AL  => self.rax as u8 as u64,
            Register::AH  => ((self.rax >> 8) as u8) as u64,

            Register::RBX => self.rbx,
            Register::EBX => self.rbx as u32 as u64,
            Register::BX  => self.rbx as u16 as u64,
            Register::BL  => self.rbx as u8 as u64,
            Register::BH  => ((self.rbx >> 8) as u8) as u64,

            Register::RCX => self.rcx,
            Register::ECX => self.rcx as u32 as u64,
            Register::CX  => self.rcx as u16 as u64,
            Register::CL  => self.rcx as u8 as u64,
            Register::CH  => ((self.rcx >> 8) as u8) as u64,

            Register::RDX => self.rdx,
            Register::EDX => self.rdx as u32 as u64,
            Register::DX  => self.rdx as u16 as u64,
            Register::DL  => self.rdx as u8 as u64,
            Register::DH  => ((self.rdx >> 8) as u8) as u64,

            Register::RSP => self.rsp,
            Register::ESP => self.rsp as u32 as u64,
            Register::SP  => self.rsp as u16 as u64,
            Register::SPL => self.rsp as u8 as u64,

            Register::RBP => self.rbp,
            Register::EBP => self.rbp as u32 as u64,
            Register::BP  => self.rbp as u16 as u64,
            Register::BPL => self.rbp as u8 as u64,

            Register::RSI => self.rsi,
            Register::ESI => self.rsi as u32 as u64,
            Register::SI  => self.rsi as u16 as u64,
            Register::SIL => self.rsi as u8 as u64,

            Register::RDI => self.rdi,
            Register::EDI => self.rdi as u32 as u64,
            Register::DI  => self.rdi as u16 as u64,
            Register::DIL => self.rdi as u8 as u64,

            Register::R8  => self.r8,
            Register::R8D => self.r8 as u32 as u64,
            Register::R8W => self.r8 as u16 as u64,
            Register::R8B => self.r8 as u8 as u64,

            Register::R9  => self.r9,
            Register::R9D => self.r9 as u32 as u64,
            Register::R9W => self.r9 as u16 as u64,
            Register::R9B => self.r9 as u8 as u64,

            Register::R10  => self.r10,
            Register::R10D => self.r10 as u32 as u64,
            Register::R10W => self.r10 as u16 as u64,
            Register::R10B => self.r10 as u8 as u64,

            Register::R11  => self.r11,
            Register::R11D => self.r11 as u32 as u64,
            Register::R11W => self.r11 as u16 as u64,
            Register::R11B => self.r11 as u8 as u64,

            Register::R12  => self.r12,
            Register::R12D => self.r12 as u32 as u64,
            Register::R12W => self.r12 as u16 as u64,
            Register::R12B => self.r12 as u8 as u64,

            Register::R13  => self.r13,
            Register::R13D => self.r13 as u32 as u64,
            Register::R13W => self.r13 as u16 as u64,
            Register::R13B => self.r13 as u8 as u64,

            Register::R14  => self.r14,
            Register::R14D => self.r14 as u32 as u64,
            Register::R14W => self.r14 as u16 as u64,
            Register::R14B => self.r14 as u8 as u64,

            Register::R15  => self.r15,
            Register::R15D => self.r15 as u32 as u64,
            Register::R15W => self.r15 as u16 as u64,
            Register::R15B => self.r15 as u8 as u64,

            Register::RIP => panic!("Cannot get RIP"),
            _ => panic!("Register not implemented: {:?}", reg),
        }
    }


    pub fn set(&mut self, reg: Register, value: u64) {

        match reg {

            Register::RAX => self.rax = value,
            Register::EAX => self.rax = value & 0xFFFFFFFF,
            Register::AX  => self.rax = (self.rax & !0xFFFF) | (value & 0xFFFF),
            Register::AH  => self.rax = (self.rax & !0xFF00) | ((value & 0xFF) << 8),
            Register::AL  => self.rax = (self.rax & !0xFF) | (value & 0xFF),

            Register::RBX => self.rbx = value,
            Register::EBX => self.rbx = value & 0xFFFFFFFF,
            Register::BX  => self.rbx = (self.rbx & !0xFFFF) | (value & 0xFFFF),
            Register::BH  => self.rbx = (self.rbx & !0xFF00) | ((value & 0xFF) << 8),
            Register::BL  => self.rbx = (self.rbx & !0xFF) | (value & 0xFF),

            // RCX family
            Register::RCX => self.rcx = value,
            Register::ECX => self.rcx = value & 0xFFFFFFFF,
            Register::CX  => self.rcx = (self.rcx & 0xFFFFFFFFFFFF0000) | (value & 0xFFFF),
            Register::CH  => self.rcx = (self.rcx & 0xFFFFFFFFFFFF00FF) | ((value & 0xFF) << 8),
            Register::CL  => self.rcx = (self.rcx & 0xFFFFFFFFFFFFFF00) | (value & 0xFF),

            // RDX family
            Register::RDX => self.rdx = value,
            Register::EDX => self.rdx = value & 0xFFFFFFFF,
            Register::DX  => self.rdx = (self.rdx & 0xFFFFFFFFFFFF0000) | (value & 0xFFFF),
            Register::DH  => self.rdx = (self.rdx & 0xFFFFFFFFFFFF00FF) | ((value & 0xFF) << 8),
            Register::DL  => self.rdx = (self.rdx & 0xFFFFFFFFFFFFFF00) | (value & 0xFF),

            // RSP family
            Register::RSP => self.rsp = value,
            Register::ESP => self.rsp = value & 0xFFFFFFFF,
            Register::SP  => self.rsp = (self.rsp & 0xFFFFFFFFFFFF0000) | (value & 0xFFFF),
            Register::SPL => self.rsp = (self.rsp & 0xFFFFFFFFFFFFFF00) | (value & 0xFF),

            // RBP family
            Register::RBP => self.rbp = value,
            Register::EBP => self.rbp = value & 0xFFFFFFFF,
            Register::BP  => self.rbp = (self.rbp & 0xFFFFFFFFFFFF0000) | (value & 0xFFFF),
            Register::BPL => self.rbp = (self.rbp & 0xFFFFFFFFFFFFFF00) | (value & 0xFF),

            // RSI family
            Register::RSI => self.rsi = value,
            Register::ESI => self.rsi = value & 0xFFFFFFFF,
            Register::SI  => self.rsi = (self.rsi & 0xFFFFFFFFFFFF0000) | (value & 0xFFFF),
            Register::SIL => self.rsi = (self.rsi & 0xFFFFFFFFFFFFFF00) | (value & 0xFF),

            // RDI family
            Register::RDI => self.rdi = value,
            Register::EDI => self.rdi = value & 0xFFFFFFFF,
            Register::DI  => self.rdi = (self.rdi & 0xFFFFFFFFFFFF0000) | (value & 0xFFFF),
            Register::DIL => self.rdi = (self.rdi & 0xFFFFFFFFFFFFFF00) | (value & 0xFF),

            // R8 family
            Register::R8 => self.r8 = value,
            Register::R8D => self.r8 = value & 0xFFFFFFFF,
            Register::R8W => self.r8 = (self.r8 & 0xFFFFFFFFFFFF0000) | (value & 0xFFFF),
            Register::R8B => self.r8 = (self.r8 & 0xFFFFFFFFFFFFFF00) | (value & 0xFF),

            // R9 family
            Register::R9 => self.r9 = value,
            Register::R9D => self.r9 = value & 0xFFFFFFFF,
            Register::R9W => self.r9 = (self.r9 & 0xFFFFFFFFFFFF0000) | (value & 0xFFFF),
            Register::R9B => self.r9 = (self.r9 & 0xFFFFFFFFFFFFFF00) | (value & 0xFF),

            // R10 family
            Register::R10 => self.r10 = value,
            Register::R10D => self.r10 = value & 0xFFFFFFFF,
            Register::R10W => self.r10 = (self.r10 & 0xFFFFFFFFFFFF0000) | (value & 0xFFFF),
            Register::R10B => self.r10 = (self.r10 & 0xFFFFFFFFFFFFFF00) | (value & 0xFF),

            // R11 family
            Register::R11 => self.r11 = value,
            Register::R11D => self.r11 = value & 0xFFFFFFFF,
            Register::R11W => self.r11 = (self.r11 & 0xFFFFFFFFFFFF0000) | (value & 0xFFFF),
            Register::R11B => self.r11 = (self.r11 & 0xFFFFFFFFFFFFFF00) | (value & 0xFF),

            // R12 family
            Register::R12 => self.r12 = value,
            Register::R12D => self.r12 = value & 0xFFFFFFFF,
            Register::R12W => self.r12 = (self.r12 & 0xFFFFFFFFFFFF0000) | (value & 0xFFFF),
            Register::R12B => self.r12 = (self.r12 & 0xFFFFFFFFFFFFFF00) | (value & 0xFF),

            // R13 family
            Register::R13 => self.r13 = value,
            Register::R13D => self.r13 = value & 0xFFFFFFFF,
            Register::R13W => self.r13 = (self.r13 & 0xFFFFFFFFFFFF0000) | (value & 0xFFFF),
            Register::R13B => self.r13 = (self.r13 & 0xFFFFFFFFFFFFFF00) | (value & 0xFF),

            // R14 family
            Register::R14 => self.r14 = value,
            Register::R14D => self.r14 = value & 0xFFFFFFFF,
            Register::R14W => self.r14 = (self.r14 & 0xFFFFFFFFFFFF0000) | (value & 0xFFFF),
            Register::R14B => self.r14 = (self.r14 & 0xFFFFFFFFFFFFFF00) | (value & 0xFF),

            // R15 family
            Register::R15 => self.r15 = value,
            Register::R15D => self.r15 = value & 0xFFFFFFFF,
            Register::R15W => self.r15 = (self.r15 & 0xFFFFFFFFFFFF0000) | (value & 0xFFFF),
            Register::R15B => self.r15 = (self.r15 & 0xFFFFFFFFFFFFFF00) | (value & 0xFF),

            // RIP
            Register::RIP => panic!("Cannot set RIP"),
            _ => panic!("Unhandled")
        }

        debug!("{:?} -> 0x{:X}", reg, value);
    }

  
}
