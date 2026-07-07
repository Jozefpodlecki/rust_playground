use core::fmt;
use core::ptr;
use ntapi::ntmmapi::*;
use ntapi::ntpsapi::*;
use winapi::shared::ntstatus::STATUS_MEMORY_NOT_ALLOCATED;
use winapi::shared::ntstatus::STATUS_NOT_COMMITTED;
use winapi::um::winnt::PVOID;
use winapi::shared::ntdef::*;
use winapi::shared::ntstatus::STATUS_SUCCESS;
use winapi::um::winnt::*;

#[derive(Debug)]
pub enum ThreadStackError {
    AllocationFailed(NTSTATUS),
    CommitFailed(NTSTATUS),
    GuardPageFailed(NTSTATUS),
    ReleaseFailed(NTSTATUS),
}

pub struct ThreadStack {
    base: *mut u8,
    size: usize
}

impl ThreadStack {
    const PAGE: usize = 4096;
    /// Creates a new thread stack with a guard page at the bottom (lowest address).
    /// 
    /// Stack layout (x64, stack grows DOWNWARD):
    /// ```
    /// HIGH ADDRESSES
    /// +---------------------------+ <-- base + size (top of stack)
    /// |                           |
    /// |    COMMITTED STACK        |   <- Usable stack space
    /// |    (PAGE_READWRITE)       |
    /// |                           |
    /// +---------------------------+ <-- base + PAGE_SIZE
    /// |    GUARD PAGE             |   <- Guard page at bottom (detects overflow)
    /// |    (PAGE_GUARD)           |
    /// +---------------------------+ <-- base (bottom of stack)
    /// LOW ADDRESSES
    /// ```
    pub fn new(stack_size: usize) -> Result<Self, ThreadStackError> {
        let min_size = 2 * Self::PAGE;
        let mut actual_size = if stack_size < min_size { min_size } else { stack_size };
        
        unsafe {
            let mut base: PVOID = ptr::null_mut();
            let mut region_size = actual_size;

            let status = NtAllocateVirtualMemory(
                NtCurrentProcess,
                &mut base,
                0,
                &mut region_size,
                MEM_RESERVE,
                PAGE_READWRITE,
            );

            if status != STATUS_SUCCESS {
                return Err(ThreadStackError::AllocationFailed(status));
            }

            let commit_size = actual_size - Self::PAGE;
            let mut commit_base = base.add(Self::PAGE);
            let mut commit_region_size = commit_size;
            
            let status = NtAllocateVirtualMemory(
                NtCurrentProcess,
                &mut commit_base,
                0,
                &mut commit_region_size,
                MEM_COMMIT,
                PAGE_READWRITE,
            );

            if status != STATUS_SUCCESS {
                let _ = NtAllocateVirtualMemory(
                    NtCurrentProcess,
                    &mut base,
                    0,
                    &mut actual_size,
                    MEM_RELEASE,
                    PAGE_READWRITE,
                );
                return Err(ThreadStackError::CommitFailed(status));
            }

            let mut guard_addr = base;
            let mut guard_size = Self::PAGE;
            
            let status = NtAllocateVirtualMemory(
                NtCurrentProcess,
                &mut guard_addr,
                0,
                &mut guard_size,
                MEM_COMMIT,
                PAGE_READWRITE | PAGE_GUARD,
            );

            if status != STATUS_SUCCESS {
                let _ = NtAllocateVirtualMemory(
                    NtCurrentProcess,
                    &mut base,
                    0,
                    &mut actual_size,
                    MEM_RELEASE,
                    PAGE_READWRITE,
                );
                return Err(ThreadStackError::GuardPageFailed(status));
            }

            Ok(Self {
                base: base as _,
                size: actual_size,
            })
        }
    }

    pub fn top(&self) -> PVOID {
        unsafe { self.base.add(self.size) as PVOID }
    }

    pub fn bottom(&self) -> PVOID {
        self.base as _
    }

    pub fn usable_start(&self) -> PVOID {
        unsafe { self.base.add(Self::PAGE) as PVOID }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn base(&self) -> PVOID {
        self.base as _
    }

    pub fn to_initial_teb(&self) -> INITIAL_TEB {
        INITIAL_TEB {
            OldInitialTeb: INITIAL_TEB_OldInitialTeb {
                OldStackBase: ptr::null_mut(),
                OldStackLimit: ptr::null_mut(),
            },
            StackBase: unsafe { self.base.add(self.size - (5 * Self::PAGE)) as _  },
            StackLimit: self.base as _,
            StackAllocationBase: self.base as _,
        }
    }

    pub fn rsp(&self) -> u64 {
        let top = unsafe { self.base.add(self.size - (5 * Self::PAGE)) } as usize;
        (top & !0xF) as u64
    }

    pub fn free(self) -> Result<(), ThreadStackError> {
        if self.base.is_null() {
            return Ok(());
        }

        unsafe {
            let mut base = self.base as PVOID;
            let mut region_size =  0;

            let status = NtFreeVirtualMemory(
                NtCurrentProcess,
                &mut base,
                &mut region_size,
                MEM_RELEASE,
            );

            if status == STATUS_NOT_COMMITTED || status == STATUS_MEMORY_NOT_ALLOCATED || status == STATUS_SUCCESS {
                Ok(())
            } else {
                Err(ThreadStackError::ReleaseFailed(status))
            }
        }
    }

    pub fn is_freed(&self) -> bool {
        self.base.is_null()
    }
}

impl fmt::Display for ThreadStackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThreadStackError::AllocationFailed(status) => write!(f, "Stack allocation failed: 0x{:X}", status),
            ThreadStackError::CommitFailed(status) => write!(f, "Stack commit failed: 0x{:X}", status),
            ThreadStackError::GuardPageFailed(status) => write!(f, "Guard page allocation failed: 0x{:X}", status),
            ThreadStackError::ReleaseFailed(status) => write!(f, "Stack release failed: 0x{:X}", status),
        }
    }
}

impl fmt::Display for ThreadStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ThreadStack {{ base: {:p}, size: 0x{:X}, top: {:p}, usable_start: {:p}, rsp: 0x{:0X} }}",
            self.base(),
            self.size,
            self.top(),
            self.usable_start(),
            self.rsp()
        )
    }
}

// pub fn debug_memory_state(addr: PVOID) {
//     unsafe {
//         let mut mbi: MEMORY_BASIC_INFORMATION = core::mem::zeroed();
//         let mut length = core::mem::size_of::<MEMORY_BASIC_INFORMATION>();
        
//         let status = NtQueryVirtualMemory(
//             NtCurrentProcess,
//             addr,
//             MemoryBasicInformation,
//             &mut mbi as *mut _ as PVOID,
//             length,
//             &mut length,
//         );
        
//         if status == STATUS_SUCCESS {
//             println!("Address: {:p}", addr);
//             println!("  BaseAddress: {:p}", mbi.BaseAddress);
//             println!("  AllocationBase: {:p}", mbi.AllocationBase);
//             println!("  RegionSize: 0x{:X}", mbi.RegionSize);
//             println!("  State: 0x{:X}", mbi.State);
//             println!("  Protect: 0x{:X}", mbi.Protect);
//             println!("  Type: 0x{:X}", mbi.Type);
            
//             let state_str = match mbi.State {
//                 MEM_COMMIT => "MEM_COMMIT",
//                 MEM_RESERVE => "MEM_RESERVE",
//                 MEM_FREE => "MEM_FREE",
//                 _ => "UNKNOWN",
//             };
//             println!("  State: {}", state_str);
//         } else {
//             println!("NtQueryVirtualMemory failed: 0x{:X}", status);
//         }
//     }
// }