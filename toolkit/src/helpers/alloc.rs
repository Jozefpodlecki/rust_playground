use core::fmt::{self, Display, Formatter};

use ntapi::{ntexapi::NtDelayExecution, ntmmapi::{MemoryBasicInformation}, ntpebteb::PEB, ntpsapi::NtCurrentProcess, ntrtl::{HEAP_INFORMATION, RTL_USER_PROCESS_PARAMETERS}};
use winapi::{ctypes::c_void, shared::ntdef::{HANDLE, LIST_ENTRY, NT_SUCCESS, NTSTATUS, PVOID, UNICODE_STRING}, um::winnt::{LARGE_INTEGER, MEM_COMMIT, MEM_RESERVE, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE_READWRITE, RTL_RUN_ONCE}};

use crate::{MemoryRegionIterator, U16CStackString, print, println, syscalls::{NtAllocateVirtualMemory, NtWriteVirtualMemory}, types::{ByteBlock, HEAP}};

pub struct ProcessMemoryAlloc;

impl ProcessMemoryAlloc {
     pub fn allocate(handle: *mut c_void, size: usize) -> Result<*mut c_void, NTSTATUS> {
        let mut base_address: *mut c_void = core::ptr::null_mut();
        let mut region_size = size;
        
        let status = unsafe {
            NtAllocateVirtualMemory(
                handle,
                &mut base_address,
                0,
                &mut region_size,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_EXECUTE_READWRITE,
            )
        };

        if !NT_SUCCESS(status) {
            return Err(status);
        }

        Ok(base_address)
    }

    pub fn allocate_and_write_bytes(handle: *mut c_void, data: &[u8]) -> Result<*mut c_void, NTSTATUS> {
        let mut base_address: *mut c_void = core::ptr::null_mut();
        let mut region_size = data.len();
        
        let status = unsafe {
            NtAllocateVirtualMemory(
                handle,
                &mut base_address,
                0,
                &mut region_size,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_EXECUTE_READWRITE,
            )
        };

        if status < 0 {
            return Err(status);
        }
        
        let mut bytes_written = 0;
        let status = unsafe {
            NtWriteVirtualMemory(
                handle,
                base_address as *mut _,
                data.as_ptr() as *mut _,
                data.len(),
                &mut bytes_written,
            )
        };
        
        if status < 0 {
            return Err(status);
        }

        Ok(base_address)
    }
}
