use std::u64;

use anyhow::{bail, Result};
use decompiler_lib::decompiler::types::Register;
use log::debug;

#[derive(Debug, Default, Clone)]
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

    #[inline]
    fn lo8(x: u64) -> u64 { x & 0xFF }
    #[inline]
    fn hi8(x: u64) -> u64 { (x >> 8) & 0xFF }
    #[inline]
    fn lo16(x: u64) -> u64 { x & 0xFFFF }
    #[inline]
    fn lo32(x: u64) -> u64 { x & 0xFFFF_FFFF }

    pub fn get(&self, reg: Register) -> u64 {
        match reg {
            // RAX family
            Register::RAX | Register::EAX | Register::AX | Register::AH | Register::AL => self.rax,
            // RBX family
            Register::RBX | Register::EBX | Register::BX | Register::BH | Register::BL => self.rbx,
            // RCX family
            Register::RCX | Register::ECX | Register::CX | Register::CH | Register::CL => self.rcx,
            // RDX family
            Register::RDX | Register::EDX | Register::DX | Register::DH | Register::DL => self.rdx,
            // RSP family
            Register::RSP | Register::ESP | Register::SP | Register::SPL => self.rsp,
            // RBP family
            Register::RBP | Register::EBP | Register::BP | Register::BPL => self.rbp,
            // RSI family
            Register::RSI | Register::ESI | Register::SI | Register::SIL => self.rsi,
            // RDI family
            Register::RDI | Register::EDI | Register::DI | Register::DIL => self.rdi,
            // R8 family
            Register::R8 | Register::R8D | Register::R8W | Register::R8B => self.r8,
            // R9 family
            Register::R9 | Register::R9D | Register::R9W | Register::R9B => self.r9,
            // R10 family
            Register::R10 | Register::R10D | Register::R10W | Register::R10B => self.r10,
            // R11 family
            Register::R11 | Register::R11D | Register::R11W | Register::R11B => self.r11,
            // R12 family
            Register::R12 | Register::R12D | Register::R12W | Register::R12B => self.r12,
            // R13 family
            Register::R13 | Register::R13D | Register::R13W | Register::R13B => self.r13,
            // R14 family
            Register::R14 | Register::R14D | Register::R14W | Register::R14B => self.r14,
            // R15 family
            Register::R15 | Register::R15D | Register::R15W | Register::R15B => self.r15,

            Register::RIP => panic!("Cannot get RIP"),
            _ => panic!("Register not implemented: {:?}", reg),
        }
    }

    pub fn set(&mut self, reg: Register, value: u64) {
        debug!("{:?} -> 0x{:X}", reg, value);

        match reg {
            // RAX family
            Register::RAX | Register::EAX | Register::AX | Register::AH | Register::AL => self.rax = value,
            // RBX family
            Register::RBX | Register::EBX | Register::BX | Register::BH | Register::BL => self.rbx = value,
            // RCX family
            Register::RCX | Register::ECX | Register::CX | Register::CH | Register::CL => self.rcx = value,
            // RDX family
            Register::RDX | Register::EDX | Register::DX | Register::DH | Register::DL => self.rdx = value,
            // RSP family
            Register::RSP | Register::ESP | Register::SP | Register::SPL => self.rsp = value,
            // RBP family
            Register::RBP | Register::EBP | Register::BP | Register::BPL => self.rbp = value,
            // RSI family
            Register::RSI | Register::ESI | Register::SI | Register::SIL => self.rsi = value,
            // RDI family
            Register::RDI | Register::EDI | Register::DI | Register::DIL => self.rdi = value,
            // R8 family
            Register::R8 | Register::R8D | Register::R8W | Register::R8B => self.r8 = value,
            // R9 family
            Register::R9 | Register::R9D | Register::R9W | Register::R9B => self.r9 = value,
            // R10 family
            Register::R10 | Register::R10D | Register::R10W | Register::R10B => self.r10 = value,
            // R11 family
            Register::R11 | Register::R11D | Register::R11W | Register::R11B => self.r11 = value,
            // R12 family
            Register::R12 | Register::R12D | Register::R12W | Register::R12B => self.r12 = value,
            // R13 family
            Register::R13 | Register::R13D | Register::R13W | Register::R13B => self.r13 = value,
            // R14 family
            Register::R14 | Register::R14D | Register::R14W | Register::R14B => self.r14 = value,
            // R15 family
            Register::R15 | Register::R15D | Register::R15W | Register::R15B => self.r15 = value,

            Register::RIP => panic!("Cannot set RIP"),
            _ => panic!("Register not implemented: {:?}", reg),
        }
    }
}