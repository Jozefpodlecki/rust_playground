use core::ptr::{null, null_mut};

use ntapi::ntrtl::*;
use winapi::shared::{
    basetsd::SIZE_T,
    ntdef::{PVOID, PWSTR, UNICODE_STRING},
};

use crate::{types::*, utils::*};

pub struct DynamicEnvironment {
    ptr: EnvironmentPtr,
}

impl Environment for DynamicEnvironment {
    type Key = &'static str;
    type Value = &'static str;
    type ViewValue<'a> = EnvironmentValue<'a>;

    fn set(&mut self, name: &Self::Key, value: &Self::Value) -> Result<(), EnvironmentError> {
        let mut name_uc = UnicodeStringWrapper::new();
        let mut value_uc = UnicodeStringWrapper::new();
        
        let name_cstr = core::ffi::CStr::from_bytes_with_nul(name.as_bytes())
            .map_err(|_| EnvironmentError::InvalidName)?;
        
        let value_cstr = core::ffi::CStr::from_bytes_with_nul(value.as_bytes())
            .map_err(|_| EnvironmentError::InvalidValue)?;
        
        unsafe {
            if !name_uc.from_asciiz(name_cstr.as_ptr() as _) {
                return Err(EnvironmentError::InvalidName);
            }
            
            if !value_uc.from_asciiz(value_cstr.as_ptr() as _) {
                return Err(EnvironmentError::InvalidValue);
            }
            
            let status = RtlSetEnvironmentVariable(
                &mut self.ptr.as_ptr(),
                name_uc.as_mut_ptr(),
                value_uc.as_mut_ptr(),
            );
            
            if status == 0 {
                Ok(())
            } else {
                Err(EnvironmentError::NtStatus(status))
            }
        }
    }

    fn get<'a>(&'a mut self, name: &Self::Key) -> Option<Self::ViewValue<'a>> {
        let mut name_uc = UnicodeStringWrapper::new();
        let mut value_uc = UnicodeStringWrapper::new();
        
        let name_cstr = core::ffi::CStr::from_bytes_with_nul(name.as_bytes()).ok()?;
        
        unsafe {
            if !name_uc.from_asciiz(name_cstr.as_ptr() as _) {
                return None;
            }
            
            let status = RtlQueryEnvironmentVariable_U(
                self.ptr.as_ptr(),
                name_uc.as_mut_ptr(),
                value_uc.as_mut_ptr(),
            );
            
            if status == 0 {
                let slice = value_uc.as_slice_leaked();
                Some(EnvironmentValue::new(slice))
            } else {
                None
            }
        }
    }

    fn remove(&mut self, name: &Self::Key) -> Result<(), EnvironmentError> {
        let name_wide: heapless::Vec<u16, 256> = name.encode_utf16().chain(Some(0)).collect();
        let mut name_uc = UNICODE_STRING {
            Length: ((name_wide.len() - 1) * 2) as u16,
            MaximumLength: (name_wide.len() * 2) as u16,
            Buffer: name_wide.as_ptr() as PWSTR,
        };
        
        unsafe {
            let status = RtlSetEnvironmentVariable(
                &mut self.ptr.as_ptr(),
                &mut name_uc,
                core::ptr::null_mut(),
            );
            
            if status == 0 {
                Ok(())
            } else {
                Err(EnvironmentError::NtStatus(status))
            }
        }
    }

    fn as_slice(&self) -> &[u16] {
        if self.ptr.as_ptr().is_null() {
            return &[];
        }
        
        let mut pos: isize = 0;
        let ptr = self.ptr.as_const_ptr();
        unsafe {
            while *ptr.offset(pos) != 0 || *ptr.offset(pos + 1) != 0 {
                pos += 1;
            }
            pos += 2;
        }
        
        unsafe { core::slice::from_raw_parts(ptr, pos as usize) }
    }

    fn as_raw_ptr(&self) -> *mut winapi::ctypes::c_void {
        self.ptr.as_ptr()
    }

    fn destroy(self) -> Result<(), EnvironmentError> {
        Ok(())
    }
    
    fn is_empty(&self) -> bool {
        is_empty_block(self.as_slice())
    }
    
    fn len(&self) -> usize {
        count_variables(self.as_slice())
    }

    fn clear(&mut self) -> Result<(), EnvironmentError> {
        unsafe {
            let mut new_env = core::ptr::null_mut();
            let status = RtlCreateEnvironmentEx(null_mut(), &mut new_env, 0);
            if status != 0 {
                return Err(EnvironmentError::NtStatus(status));
            }
            
            self.ptr = EnvironmentPtr::from_existing(new_env);
            
            Ok(())
        }
    }
    
    fn capacity(&self) -> usize {
        self.as_slice().len()
    }
}

impl DynamicEnvironment {
    pub fn set_current(&self) {
        unsafe {
            RtlSetCurrentEnvironment(self.ptr.as_ptr(), null_mut());
        }
    }

    pub fn new() -> Result<Self, EnvironmentError> {
        unsafe {
            let ptr = EnvironmentPtr::new()
                .map_err(|status| EnvironmentError::NtStatus(status))?;
            Ok(Self { ptr })
        }
    }

    pub fn from_source(source: PVOID) -> Result<Self, EnvironmentError> {
        let mut env_ptr = null_mut();
        unsafe { 
            let status = RtlCreateEnvironmentEx(source, &mut env_ptr, 0);
            if status != 0 {
                return Err(EnvironmentError::NtStatus(status));
            }
        }
        unsafe {
            Ok(Self {
                ptr: EnvironmentPtr::from_existing(env_ptr)
            })
        }
    }

    pub fn from_ptr(data: *const u16) -> Result<Self, EnvironmentError> {
        unsafe {
            Ok(Self {
                ptr: EnvironmentPtr::from_existing(data as *mut _)
            })
        }
    }

    pub fn iter(&self) -> EnvironmentIter<'_> {
        EnvironmentIter::new(self.as_slice())
    }
}