use anyhow::{bail, Result};
use crate::memory::Memory;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Reg {
    Rax, Rcx, Rdx, Rbx, Rsp, Rbp, Rsi, Rdi,
    R8, R9, R10, R11, R12, R13, R14, R15,
}


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
    pub fn new() -> Self {
        Self {
            rax: 0,
            rcx: 0,
            rdx: 0,
            rbx: 0,
            rsp: 0,
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

    pub fn get(&self, reg: Reg) -> u64 {
        match reg {
            Reg::Rax => self.rax,
            Reg::Rcx => self.rcx,
            Reg::Rdx => self.rdx,
            Reg::Rbx => self.rbx,
            Reg::Rsp => self.rsp,
            Reg::Rbp => self.rbp,
            Reg::Rsi => self.rsi,
            Reg::Rdi => self.rdi,
            Reg::R8 => self.r8,
            Reg::R9 => self.r9,
            Reg::R10 => self.r10,
            Reg::R11 => self.r11,
            Reg::R12 => self.r12,
            Reg::R13 => self.r13,
            Reg::R14 => self.r14,
            Reg::R15 => self.r15,
        }
    }

    pub fn set(&mut self, reg: Reg, val: u64) {
        match reg {
            Reg::Rax => self.rax = val,
            Reg::Rcx => self.rcx = val,
            Reg::Rdx => self.rdx = val,
            Reg::Rbx => self.rbx = val,
            Reg::Rsp => self.rsp = val,
            Reg::Rbp => self.rbp = val,
            Reg::Rsi => self.rsi = val,
            Reg::Rdi => self.rdi = val,
            Reg::R8 => self.r8 = val,
            Reg::R9 => self.r9 = val,
            Reg::R10 => self.r10 = val,
            Reg::R11 => self.r11 = val,
            Reg::R12 => self.r12 = val,
            Reg::R13 => self.r13 = val,
            Reg::R14 => self.r14 = val,
            Reg::R15 => self.r15 = val,
        }
    }
}