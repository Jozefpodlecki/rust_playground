
use core::{fmt, ops::Deref};

use ntapi::{ntpebteb::PEB, ntpsapi::NtCurrentPeb, ntrtl::RTL_USER_PROCESS_PARAMETERS};
use winapi::shared::ntdef::UNICODE_STRING;

use crate::{peb::exec_path::ExecutablePath, types::{PVOID, Utf16Path}};

mod exec_path;
mod params;

pub struct ProcessEnvironmentBlock(*mut PEB);

impl ProcessEnvironmentBlock {
    pub fn current_process() -> Self {
        let peb: *mut PEB = unsafe { NtCurrentPeb() };
        Self(peb)
    }

    pub fn as_ptr(&self) -> *mut PEB {
        self.0
    }
    
    pub fn image_base(&self) -> PVOID {
        unsafe { (*self.0).ImageBaseAddress as _ }
    }

    pub fn standard_output(&self) -> Option<PVOID> {
        let params = unsafe { &*(*self.0).ProcessParameters };
        
        if params.StandardOutput.is_null() {
            return None
        }

        Some(params.StandardOutput as _)
    }

    pub const fn executable_path(&self) -> ExecutablePath {
        let params = unsafe { &*(*self.0).ProcessParameters };
        let image_path: UNICODE_STRING = params.ImagePathName;
        let path = Utf16Path::new(image_path.Buffer, (image_path.Length / 2) as usize);
        ExecutablePath::new(path)
    }

    pub fn process_parameters_raw(&self) -> &RTL_USER_PROCESS_PARAMETERS {
        unsafe { &*(*self.0).ProcessParameters }
    }
}