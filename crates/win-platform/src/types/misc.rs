#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

use winapi::um::winnt::LARGE_INTEGER;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KCONTINUE_TYPE {
    KCONTINUE_UNWIND = 0,
    KCONTINUE_RESUME = 1,
    KCONTINUE_LONGJUMP = 2,
    KCONTINUE_SET = 3,
    KCONTINUE_LAST = 4,
}

pub const KCONTINUE_FLAG_TEST_ALERT: u32 = 0x00000001;
pub const KCONTINUE_FLAG_DELIVER_APC: u32 = 0x00000002;
pub type PKContinueArgument = *mut KContinueArgument;

#[repr(C)]
pub struct KContinueArgument {
    pub ContinueType: KCONTINUE_TYPE,
    pub ContinueFlags: u32,
    pub Reserved: [u64; 2],
}

impl KContinueArgument {
    pub const fn resume() -> Self {
        Self {
            ContinueType: KCONTINUE_TYPE::KCONTINUE_RESUME,
            ContinueFlags: 0,
            Reserved: [0, 0],
        }
    }
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct FILE_PIPE_WAIT_FOR_BUFFER {
    pub Timeout: LARGE_INTEGER,
    pub NameLength: u32,
    pub TimeoutSpecified: u8,
    pub Padding: u8,
    pub Name: [u16; 1],
}