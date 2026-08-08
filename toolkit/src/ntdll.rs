use ntapi::ntpsapi::NtCurrentProcess;
use winapi::{shared::ntdef::{NTSTATUS, PVOID}, um::minwinbase::PTHREAD_START_ROUTINE};

use crate::{MemoryRegionIterator, println};


pub struct NtDll(PVOID);

//  

impl NtDll {
    pub fn from_remote_process(handle: *mut winapi::ctypes::c_void) -> Self {
        
        for mbi in MemoryRegionIterator::from_address(handle, 0x00007FF000000000u64 as _) {
            if !mbi.mapped_name.is_empty() {
                if mbi.mapped_name.contains("ntdll.dll") {
                    return Self(mbi.allocation_base())
                }
            }
        }
        
        unreachable!("ntdll not found");
    }

    pub fn from_current_process() -> Self {
        
        // for mbi in MemoryRegionIterator::new(NtCurrentProcess) {
        for mbi in MemoryRegionIterator::from_address(NtCurrentProcess, 0x00007FF000000000u64 as _) {
            if !mbi.mapped_name.is_empty() {
                if mbi.mapped_name.contains("ntdll.dll") {
                    return Self(mbi.allocation_base())
                }
            }
        }
        
        unreachable!("ntdll not found");
    }

    pub fn base(&self) -> PVOID {
        self.0
    }

    // pub fn vectored_handler_list(&self) -> PVOID {
    //     unsafe { self.0.add(0x1E9578) }
    // }

    // #[allow(non_snake_case)]
    // pub fn RtlExitUserThread(&self) -> unsafe extern "system" fn(Code: NTSTATUS) {
    //     let offset = 0x87C60;
    //     let func_ptr = self.0 as usize + offset;
        
    //     unsafe { core::mem::transmute(func_ptr) }
    // }

    // #[allow(non_snake_case)]
    // pub fn RtlUserThreadStart(&self) -> unsafe extern "system" fn(Function: PTHREAD_START_ROUTINE, Parameter: PVOID) -> NTSTATUS {
    //     let offset = 0x87BF0;
    //     let func_ptr = self.0 as usize + offset;
        
    //     unsafe { core::mem::transmute(func_ptr) }
    // }
}