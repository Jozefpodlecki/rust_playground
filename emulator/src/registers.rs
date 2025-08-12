use std::u64;

use anyhow::{bail, Result};
use decompiler_lib::decompiler::types::Register;

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
        Self {
            rax: 0,
            rcx: 0,
            rdx: 0,
            rbx: 0,
            rsp,
            rbp: 0,
            rsi: 0,
            rdi: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
        }
    }

    pub fn get(&self, reg: Register) -> u64 {
        match reg {
            Register::Rax => self.rax,
            Register::Eax => self.rax,
            Register::Rbx => self.rbx,
            Register::Ebx => self.rbx,
            Register::Bx => self.rbx,
            Register::Rcx => self.rcx,
            Register::Ecx => self.rcx,
            Register::Rdx => self.rdx,
            Register::Edx => self.rdx,
            Register::Rsp => self.rsp,
            Register::Rbp => self.rbp,
            Register::Rsi => self.rsi,
            Register::Rdi => self.rdi,
            Register::Edi => self.rdi,
            Register::R8 => self.r8,
            Register::R9 => self.r9,
            Register::R10 => self.r10,
            Register::R11 => self.r11,
            Register::R12 => self.r12,
            Register::R13 => self.r13,
            Register::R14 => self.r14,
            Register::R15 => self.r15,
            Register::Rip => panic!("Invalid operation"),
            Register::Unknown(id) => panic!("Invalid register: {}", id)
        }
    }

    pub fn set(&mut self, reg: Register, val: u64) {
        let old_val = self.get(reg);
        if old_val != val {
            println!(
                "[REG] {:?}: {:#x} → {:#x}",
                reg, old_val, val
            );
        }

        match reg {
            Register::Rax => self.rax = val,
            Register::Eax => self.rax = val,
            Register::Rbx => self.rbx = val,
            Register::Ebx => self.rbx = val,
            Register::Bx => self.rbx = val,
            Register::Rcx => self.rcx = val,
            Register::Ecx => self.rcx = val,
            Register::Rdx => self.rdx = val,
            Register::Edx => self.rdx = val,
            Register::Rsp => self.rsp = val,
            Register::Rbp => self.rbp = val,
            Register::Rsi => self.rsi = val,
            Register::Rdi => self.rdi = val,
            Register::Edi => self.rdi = val,
            Register::R8 => self.r8 = val,
            Register::R9 => self.r9 = val,
            Register::R10 => self.r10 = val,
            Register::R11 => self.r11 = val,
            Register::R12 => self.r12 = val,
            Register::R13 => self.r13 = val,
            Register::R14 => self.r14 = val,
            Register::R15 => self.r15 = val,
            Register::Rip => panic!("Invalid operation"),
            Register::Unknown(id) => panic!("Invalid register: {}", id)
        }
    }
}