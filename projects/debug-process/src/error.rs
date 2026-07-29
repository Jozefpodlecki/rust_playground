use core::fmt;

use winapi::shared::ntdef::NTSTATUS;


pub enum DebuggerError {
    CouldNotCreateDebugObject(NTSTATUS),
    CouldNotSpawnProcess(NTSTATUS),
    CouldNotAttach(NTSTATUS),
    InvalidHandle,
    EventTooLarge {
        code: u32,
        size: usize,
    },
    ThreadNotFound,
    MemoryAllocation,
    CouldNotResumeThread(NTSTATUS),
    Continue(NTSTATUS),
    AlreadyAttached,
    NotAttached,
    Timeout,
    InvalidEvent(u32),
    Wait(NTSTATUS),
}

impl fmt::Display for DebuggerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DebuggerError::CouldNotCreateDebugObject(status) => {
                write!(f, "Failed to create debug object: NTSTATUS 0x{:X}", status)
            }
            DebuggerError::CouldNotSpawnProcess(status) => {
                write!(f, "Failed to spawn process: NTSTATUS 0x{:X}", status)
            }
            DebuggerError::InvalidHandle => {
                write!(f, "Invalid handle received from system call")
            }
            DebuggerError::AlreadyAttached => {
                write!(f, "Debugger is already attached to a process")
            }
            DebuggerError::NotAttached => {
                write!(f, "Debugger is not attached to any process")
            }
            DebuggerError::Timeout => {
                write!(f, "Wait for debug event timed out")
            }
            DebuggerError::Wait(status) => {
                write!(f, "Failed to wait for debug event: 0x{:X}", status)
            }
            DebuggerError::CouldNotAttach(status) => 
                write!(f, "Could not attach process: NTSTATUS 0x{:X}", status),
            DebuggerError::InvalidEvent(code) => write!(f, "Invalid event: {code}"),
            DebuggerError::ThreadNotFound => write!(f, "Main thread not found"),
            DebuggerError::CouldNotResumeThread(status) => write!(f, "Could not resume thread: NTSTATUS 0x{:X}", status),
            DebuggerError::Continue(status) => write!(f, "Could not continue: NTSTATUS 0x{:X}", status),
            DebuggerError::EventTooLarge { code, size} => write!(f, "Debug event too large for stack buffer code: 0{code:X}, size: 0{size:X}"),
            DebuggerError::MemoryAllocation => write!(f, "Memory allocation failed"),
        }
    }
}
