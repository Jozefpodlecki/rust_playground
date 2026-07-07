use ntapi::{ntpebteb::PEB, ntrtl::RTL_USER_PROCESS_PARAMETERS};
use winapi::{ctypes::c_void, shared::ntdef::UNICODE_STRING};

use crate::{U16CStackString, types::HEAP};



#[unsafe(naked)]
pub unsafe fn get_peb() -> *mut PEB {
    core::arch::naked_asm!(
        "mov rax, gs:[0x60]",
        "ret"
    );
}

pub struct ProcessEnvironmentBlock(*mut PEB);

impl ProcessEnvironmentBlock {
    pub fn current_process() -> Self {
        let peb: *mut PEB;
        unsafe {
            core::arch::asm!(
                "mov {0}, gs:[0x60]",
                out(reg) peb,
                options(nostack, readonly)
            );
        }
        Self(peb)
    }

    pub fn process_params(&self) -> *mut RTL_USER_PROCESS_PARAMETERS {
        unsafe { (*self.0).ProcessParameters }
    }
    
    pub fn image_base(&self) -> *mut c_void {
        unsafe { (*self.0).ImageBaseAddress }
    }

    pub fn process_heap(&self) -> *mut HEAP {
        unsafe {
            (*self.0).ProcessHeap as *mut HEAP
            // let raw_heap = (*self.0).ProcessHeap;
            // // The _HEAP structure starts at offset 0x10 (after the initial HEAP_ENTRY)
            // (raw_heap as usize + 0x10) as *mut HEAP
        }
    }

    pub fn executable_path(&self) -> U16CStackString<260> {
        let params = unsafe { &*(*self.0).ProcessParameters };
        let image_path: UNICODE_STRING = params.ImagePathName;
        U16CStackString::<260>::from_ptr(image_path.Buffer).unwrap()
    }

}