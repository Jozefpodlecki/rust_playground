use ntapi::ntrtl::{RtlAllocateHeap, RtlProcessHeap};
use winapi::um::winnt::{HEAP_ZERO_MEMORY, RtlCopyMemory};

use crate::{fixed::FixedEnvironment, heap::DynamicEnvironment, types::{Environment, EnvironmentError}};

pub trait DynamicEnvironmentExtensions {
    fn to_fixed<const N: usize>(&self) -> Result<FixedEnvironment<N>, EnvironmentError>;
}

pub trait FixedEnvironmentExtensions {
    fn to_heap(&self) -> Result<DynamicEnvironment, EnvironmentError>;
}

impl DynamicEnvironmentExtensions for DynamicEnvironment {
    fn to_fixed<const N: usize>(&self) -> Result<FixedEnvironment<N>, EnvironmentError> {
        let raw_ptr = self.as_raw_ptr();
        if raw_ptr.is_null() {
            return Err(EnvironmentError::InvalidValue);
        }
        
        let mut pos = 0;
        let env_ptr = raw_ptr as *const u16;
        
        unsafe {
            while *env_ptr.offset(pos) != 0 || *env_ptr.offset(pos + 1) != 0 {
                pos += 1;
            }
            pos += 2;
        }
        
        let len = pos;
        
        if len as usize > N {
            return Err(EnvironmentError::BufferFull);
        }
        
        let slice = unsafe { core::slice::from_raw_parts(env_ptr, len as usize) };
        FixedEnvironment::from_slice(slice)
    }
}


impl<const N: usize> FixedEnvironmentExtensions for FixedEnvironment<N> {
    fn to_heap(&self) -> Result<DynamicEnvironment, EnvironmentError> {
        unsafe { 
            let slice = self.as_slice();
            let ptr = RtlAllocateHeap(RtlProcessHeap(), HEAP_ZERO_MEMORY, slice.len());

            RtlCopyMemory(ptr, slice.as_ptr() as _, slice.len());
            
            DynamicEnvironment::from_ptr(ptr as _)
        }
    }
}