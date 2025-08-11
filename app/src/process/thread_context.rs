use std::io::{Read, Write};
use anyhow::Result;
use byteorder::LittleEndian;
use windows::Win32::System::Diagnostics::Debug::CONTEXT;
use byteorder::{ReadBytesExt, WriteBytesExt};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ThreadContext {
    // Control
    pub rip: u64,
    pub rsp: u64,
    pub rbp: u64,
    pub eflags: u32,
    pub mxcsr: u32,

    // General-purpose registers
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
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

    // Segment registers (optional, rarely needed)
    pub cs: u16,
    pub ds: u16,
    pub es: u16,
    pub fs: u16,
    pub gs: u16,
    pub ss: u16,

    // Debug registers (optional but useful for hardware breakpoints)
    pub dr0: u64,
    pub dr1: u64,
    pub dr2: u64,
    pub dr3: u64,
    pub dr6: u64,
    pub dr7: u64,
}

impl ThreadContext {
    pub fn new(context: CONTEXT) -> Self {
        Self {
            rip: context.Rip,
            rsp: context.Rsp,
            rbp: context.Rbp,
            eflags: context.EFlags,
            mxcsr: context.MxCsr,
            rax: context.Rax,
            rbx: context.Rbx,
            rcx: context.Rcx,
            rdx: context.Rdx,
            rsi: context.Rsi,
            rdi: context.Rdi,
            r8: context.R8,
            r9: context.R9,
            r10: context.R10,
            r11: context.R11,
            r12: context.R12,
            r13: context.R13,
            r14: context.R14,
            r15: context.R15,
            cs: context.SegCs,
            ds: context.SegDs,
            es: context.SegEs,
            fs: context.SegFs,
            gs: context.SegGs,
            ss: context.SegSs,
            dr0: context.Dr0,
            dr1: context.Dr1,
            dr2: context.Dr2,
            dr3: context.Dr3,
            dr6: context.Dr6,
            dr7: context.Dr7,
        }
    }

    pub fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        Ok(Self {
            rip: reader.read_u64::<LittleEndian>()?,
            rsp: reader.read_u64::<LittleEndian>()?,
            rbp: reader.read_u64::<LittleEndian>()?,
            eflags: reader.read_u32::<LittleEndian>()?,
            mxcsr: reader.read_u32::<LittleEndian>()?,
            rax: reader.read_u64::<LittleEndian>()?,
            rbx: reader.read_u64::<LittleEndian>()?,
            rcx: reader.read_u64::<LittleEndian>()?,
            rdx: reader.read_u64::<LittleEndian>()?,
            rsi: reader.read_u64::<LittleEndian>()?,
            rdi: reader.read_u64::<LittleEndian>()?,
            r8: reader.read_u64::<LittleEndian>()?,
            r9: reader.read_u64::<LittleEndian>()?,
            r10: reader.read_u64::<LittleEndian>()?,
            r11: reader.read_u64::<LittleEndian>()?,
            r12: reader.read_u64::<LittleEndian>()?,
            r13: reader.read_u64::<LittleEndian>()?,
            r14: reader.read_u64::<LittleEndian>()?,
            r15: reader.read_u64::<LittleEndian>()?,
            cs: reader.read_u16::<LittleEndian>()?,
            ds: reader.read_u16::<LittleEndian>()?,
            es: reader.read_u16::<LittleEndian>()?,
            fs: reader.read_u16::<LittleEndian>()?,
            gs: reader.read_u16::<LittleEndian>()?,
            ss: reader.read_u16::<LittleEndian>()?,
            dr0: reader.read_u64::<LittleEndian>()?,
            dr1: reader.read_u64::<LittleEndian>()?,
            dr2: reader.read_u64::<LittleEndian>()?,
            dr3: reader.read_u64::<LittleEndian>()?,
            dr6: reader.read_u64::<LittleEndian>()?,
            dr7: reader.read_u64::<LittleEndian>()?,
        })
    }

    pub fn encode<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u64::<LittleEndian>(self.rip)?;
        writer.write_u64::<LittleEndian>(self.rsp)?;
        writer.write_u64::<LittleEndian>(self.rbp)?;
        writer.write_u32::<LittleEndian>(self.eflags)?;
        writer.write_u32::<LittleEndian>(self.mxcsr)?;
        writer.write_u64::<LittleEndian>(self.rax)?;
        writer.write_u64::<LittleEndian>(self.rbx)?;
        writer.write_u64::<LittleEndian>(self.rcx)?;
        writer.write_u64::<LittleEndian>(self.rdx)?;
        writer.write_u64::<LittleEndian>(self.rsi)?;
        writer.write_u64::<LittleEndian>(self.rdi)?;
        writer.write_u64::<LittleEndian>(self.r8)?;
        writer.write_u64::<LittleEndian>(self.r9)?;
        writer.write_u64::<LittleEndian>(self.r10)?;
        writer.write_u64::<LittleEndian>(self.r11)?;
        writer.write_u64::<LittleEndian>(self.r12)?;
        writer.write_u64::<LittleEndian>(self.r13)?;
        writer.write_u64::<LittleEndian>(self.r14)?;
        writer.write_u64::<LittleEndian>(self.r15)?;
        writer.write_u16::<LittleEndian>(self.cs)?;
        writer.write_u16::<LittleEndian>(self.ds)?;
        writer.write_u16::<LittleEndian>(self.es)?;
        writer.write_u16::<LittleEndian>(self.fs)?;
        writer.write_u16::<LittleEndian>(self.gs)?;
        writer.write_u16::<LittleEndian>(self.ss)?;
        writer.write_u64::<LittleEndian>(self.dr0)?;
        writer.write_u64::<LittleEndian>(self.dr1)?;
        writer.write_u64::<LittleEndian>(self.dr2)?;
        writer.write_u64::<LittleEndian>(self.dr3)?;
        writer.write_u64::<LittleEndian>(self.dr6)?;
        writer.write_u64::<LittleEndian>(self.dr7)?;
        Ok(())
    }
}
