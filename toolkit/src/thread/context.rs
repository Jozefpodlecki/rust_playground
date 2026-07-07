use core::ops::{Deref, DerefMut};

use winapi::um::winnt::{CONTEXT, CONTEXT_FULL};


#[repr(align(16))]
pub struct ThreadContext(CONTEXT);

impl Deref for ThreadContext {
    type Target = CONTEXT;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ThreadContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ThreadContext {
    pub fn new() -> Self {
        Self(unsafe { core::mem::zeroed() })
    }

    pub fn set_thread_context(&mut self, rsp: u64, rip: u64, rcx: u64) {
        self.0.ContextFlags = CONTEXT_FULL;
        self.0.SegCs = 0x33;
        self.0.SegDs = 0x2b;
        self.0.SegEs = 0x2b;
        self.0.SegFs = 0x53;
        self.0.SegSs = 0x2b;
        self.0.EFlags = 0x200;
        self.0.Rax = 0;
        self.0.Rdx = 0;
        self.0.R8 = 0;
        self.0.R9 = 0;
        self.0.R10 = 0;
        self.0.R11 = 0;

        self.0.Rip = rip;
        self.0.Rcx = rcx;
        self.0.Rsp = rsp;
        self.0.Rbp = 0;
        self.0.Rsi = 0;
        self.0.Rdi = 0;
        self.0.Rbx = 0;
    }
}
