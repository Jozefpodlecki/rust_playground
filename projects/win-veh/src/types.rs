use core::fmt;

use winapi::{shared::minwindef::ULONG, um::winnt::{CONTEXT, EXCEPTION_RECORD}};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VECTORED_HANDLER_LIST {
    pub mutex_exception: *mut std::ffi::c_void,      // SRWLOCK for VEH
    pub first_exception_handler: *mut VEH_HANDLER_ENTRY,
    pub last_exception_handler: *mut VEH_HANDLER_ENTRY,
    pub mutex_continue: *mut std::ffi::c_void,       // SRWLOCK for VCH
    pub first_continue_handler: *mut VEH_HANDLER_ENTRY,
    pub last_continue_handler: *mut VEH_HANDLER_ENTRY,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LIST_ENTRY {
    pub flink: *mut LIST_ENTRY,
    pub blink: *mut LIST_ENTRY,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VEH_HANDLER_ENTRY {
    pub entry: LIST_ENTRY,     
    pub sync_refs: *mut std::ffi::c_void, // 8 bytes - SyncRefs
    pub padding: u32,
    pub rnd_upper: u32,   
    pub handler: *mut std::ffi::c_void, // Encoded handler pointer
}

#[repr(transparent)]
pub struct VehContext(pub CONTEXT);

impl fmt::Debug for VehContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ctx = &self.0;
        f.debug_struct("CONTEXT")
            .field("P1Home", &ctx.P1Home)
            .field("P2Home", &ctx.P2Home)
            .field("P3Home", &ctx.P3Home)
            .field("P4Home", &ctx.P4Home)
            .field("P5Home", &ctx.P5Home)
            .field("P6Home", &ctx.P6Home)
            .field("ContextFlags", &format!("0x{:08X}", ctx.ContextFlags))
            .field("MxCsr", &format!("0x{:08X}", ctx.MxCsr))
            .field("SegCs", &format!("0x{:04X}", ctx.SegCs))
            .field("SegDs", &format!("0x{:04X}", ctx.SegDs))
            .field("SegEs", &format!("0x{:04X}", ctx.SegEs))
            .field("SegFs", &format!("0x{:04X}", ctx.SegFs))
            .field("SegGs", &format!("0x{:04X}", ctx.SegGs))
            .field("SegSs", &format!("0x{:04X}", ctx.SegSs))
            .field("EFlags", &format!("0x{:08X}", ctx.EFlags))
            // Debug registers
            .field("Dr0", &format!("0x{:016X}", ctx.Dr0))
            .field("Dr1", &format!("0x{:016X}", ctx.Dr1))
            .field("Dr2", &format!("0x{:016X}", ctx.Dr2))
            .field("Dr3", &format!("0x{:016X}", ctx.Dr3))
            .field("Dr6", &format!("0x{:016X}", ctx.Dr6))
            .field("Dr7", &format!("0x{:016X}", ctx.Dr7))
            // General purpose registers
            .field("Rax", &format!("0x{:016X}", ctx.Rax))
            .field("Rcx", &format!("0x{:016X}", ctx.Rcx))
            .field("Rdx", &format!("0x{:016X}", ctx.Rdx))
            .field("Rbx", &format!("0x{:016X}", ctx.Rbx))
            .field("Rsp", &format!("0x{:016X}", ctx.Rsp))
            .field("Rbp", &format!("0x{:016X}", ctx.Rbp))
            .field("Rsi", &format!("0x{:016X}", ctx.Rsi))
            .field("Rdi", &format!("0x{:016X}", ctx.Rdi))
            .field("R8", &format!("0x{:016X}", ctx.R8))
            .field("R9", &format!("0x{:016X}", ctx.R9))
            .field("R10", &format!("0x{:016X}", ctx.R10))
            .field("R11", &format!("0x{:016X}", ctx.R11))
            .field("R12", &format!("0x{:016X}", ctx.R12))
            .field("R13", &format!("0x{:016X}", ctx.R13))
            .field("R14", &format!("0x{:016X}", ctx.R14))
            .field("R15", &format!("0x{:016X}", ctx.R15))
            // Instruction pointer
            .field("Rip", &format!("0x{:016X}", ctx.Rip))
            // Vector registers (show first few only to avoid spam)
            .field("VectorControl", &format!("0x{:016X}", ctx.VectorControl))
            .field("DebugControl", &format!("0x{:016X}", ctx.DebugControl))
            .field("LastBranchToRip", &format!("0x{:016X}", ctx.LastBranchToRip))
            .field("LastBranchFromRip", &format!("0x{:016X}", ctx.LastBranchFromRip))
            .field("LastExceptionToRip", &format!("0x{:016X}", ctx.LastExceptionToRip))
            .field("LastExceptionFromRip", &format!("0x{:016X}", ctx.LastExceptionFromRip))
            .finish()
    }
}

#[repr(transparent)]
pub struct VehExceptionRecord(pub EXCEPTION_RECORD);

impl fmt::Debug for VehExceptionRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rec = &self.0;
        
        let code_name = match rec.ExceptionCode {
            0xC0000005 => "EXCEPTION_ACCESS_VIOLATION",
            0xC0000094 => "EXCEPTION_INT_DIVIDE_BY_ZERO",
            0xC0000095 => "EXCEPTION_INT_OVERFLOW",
            0xC00000FD => "EXCEPTION_STACK_OVERFLOW",
            0xC0000008 => "EXCEPTION_INVALID_HANDLE",
            0x80000003 => "EXCEPTION_BREAKPOINT",
            0x80000004 => "EXCEPTION_SINGLE_STEP",
            _ => "UNKNOWN",
        };
        
        let mut s = f.debug_struct("EXCEPTION_RECORD");
        s.field("ExceptionCode", &format!("0x{:08X} ({})", rec.ExceptionCode, code_name));
        s.field("ExceptionFlags", &format!("0x{:08X}", rec.ExceptionFlags));
        s.field("NumberParameters", &rec.NumberParameters);
        s.field("ExceptionAddress", &format!("0x{:016X}", rec.ExceptionAddress as usize));
        
        // Show exception information parameters
        if rec.NumberParameters > 0 {
            let mut info = Vec::new();
            for i in 0..rec.NumberParameters as usize {
                info.push(format!("0x{:016X}", rec.ExceptionInformation[i]));
            }
            s.field("ExceptionInformation", &info);
        }
        
        s.finish()
    }
}

impl VehContext {
    pub fn context_flags_string(&self) -> String {
        let flags = self.0.ContextFlags;
        let mut parts = Vec::new();
        
        if flags & 0x00100000 != 0 { parts.push("CONTEXT_DEBUG_REGISTERS"); }
        if flags & 0x00200000 != 0 { parts.push("CONTEXT_SEGMENTS"); }
        if flags & 0x00400000 != 0 { parts.push("CONTEXT_FLOATING_POINT"); }
        if flags & 0x00800000 != 0 { parts.push("CONTEXT_AMD64"); }
        if flags & 0x01000000 != 0 { parts.push("CONTEXT_CONTROL"); }
        if flags & 0x02000000 != 0 { parts.push("CONTEXT_INTEGER"); }
        if flags & 0x04000000 != 0 { parts.push("CONTEXT_XSTATE"); }
        if flags & 0x10000000 != 0 { parts.push("CONTEXT_EXCEPTION_ACTIVE"); }
        if flags & 0x08000000 != 0 { parts.push("CONTEXT_FULL"); }
        
        if parts.is_empty() {
            format!("0x{:08X} (UNKNOWN)", flags)
        } else {
            format!("0x{:08X} ({})", flags, parts.join(" | "))
        }
    }
}
