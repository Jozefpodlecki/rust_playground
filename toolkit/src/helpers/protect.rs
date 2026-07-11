use core::fmt::{self, Display, Formatter};

use ntapi::{ntexapi::NtDelayExecution, ntmmapi::{MemoryBasicInformation, MemoryMappedFilenameInformation, NtProtectVirtualMemory, NtQueryVirtualMemory, NtReadVirtualMemory, NtWriteVirtualMemory}, ntpebteb::PEB, ntpsapi::NtCurrentProcess, ntrtl::{HEAP_INFORMATION, RTL_USER_PROCESS_PARAMETERS}};
use winapi::{ctypes::c_void, shared::ntdef::{HANDLE, LIST_ENTRY, NT_SUCCESS, NTSTATUS, PVOID, UNICODE_STRING}, um::winnt::{LARGE_INTEGER, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE_READWRITE, RTL_RUN_ONCE}};

use crate::{MemoryRegionIterator, U16CStackString, print, println, types::{ByteBlock, HEAP}};

pub struct ProcessMemoryProtector(*mut c_void);

pub struct ProcessMemoryProtectorSession {
    handle: *mut c_void,
    address: PVOID,
    size: usize,
    old_protect: u32,
}

impl ProcessMemoryProtectorSession {
    pub fn restore(self) -> Result<(), NTSTATUS> {
        let mut address = self.address;
        let mut size = self.size;
        let mut new_protect: u32 = 0;
        
        let status = unsafe {
            NtProtectVirtualMemory(
                self.handle,
                &mut address,
                &mut size,
                self.old_protect,
                &mut new_protect,
            )
        };
        
        if NT_SUCCESS(status) {
            Ok(())
        } else {
            Err(status)
        }
    }
}

impl ProcessMemoryProtector {
    pub fn current() -> Self {
        Self(NtCurrentProcess)
    }

    pub fn remote(handle: *mut c_void) -> Self {
        Self(handle)
    }

    pub fn make_writable(&self, address: PVOID, size: usize) -> Result<ProcessMemoryProtectorSession, NTSTATUS> {
        let page_size = 0x1000;
        let page_address = (address as usize & !(page_size - 1)) as PVOID;
        let region_size = ((address as usize + size - 1) & !(page_size - 1)) + page_size - (address as usize & !(page_size - 1));
        let mut region_size = region_size;
        let mut old_protect: u32 = 0;
        let mut temp_address = page_address;
        
        let status = unsafe {
            NtProtectVirtualMemory(
                self.0,
                &mut temp_address,
                &mut region_size,
                PAGE_EXECUTE_READWRITE,
                &mut old_protect,
            )
        };
        
        if !NT_SUCCESS(status) {
            return Err(status);
        }
        
        Ok(ProcessMemoryProtectorSession {
            handle: self.0,
            address: page_address,
            size: region_size,
            old_protect,
        })
    }
    
    pub fn make_writable_with(&self, address: PVOID, size: usize, protect: u32) -> Result<ProcessMemoryProtectorSession, NTSTATUS> {
        let page_size = 0x1000;
        let page_address = (address as usize & !(page_size - 1)) as PVOID;
        let region_size = ((address as usize + size - 1) & !(page_size - 1)) + page_size - (address as usize & !(page_size - 1));
        let mut region_size = region_size;
        let mut old_protect: u32 = 0;
        let mut temp_address = page_address;
        
        let status = unsafe {
            NtProtectVirtualMemory(
                self.0,
                &mut temp_address,
                &mut region_size,
                protect,
                &mut old_protect,
            )
        };
        
        if !NT_SUCCESS(status) {
            return Err(status);
        }
        
        Ok(ProcessMemoryProtectorSession {
            handle: self.0,
            address: page_address,
            size: region_size,
            old_protect,
        })
    }

    pub fn make_readonly(&self, address: PVOID, size: usize) -> Result<ProcessMemoryProtectorSession, NTSTATUS> {
        self.make_readonly_with(address, size, winapi::um::winnt::PAGE_EXECUTE_READ)
    }

    pub fn make_readonly_with(&self, address: PVOID, size: usize, protect: u32) -> Result<ProcessMemoryProtectorSession, NTSTATUS> {
        let page_size = 0x1000;
        let page_address = (address as usize & !(page_size - 1)) as PVOID;
        let region_size = ((address as usize + size - 1) & !(page_size - 1)) + page_size - (address as usize & !(page_size - 1));
        let mut region_size = region_size;
        let mut old_protect: u32 = 0;
        let mut temp_address = page_address;
        
        let status = unsafe {
            NtProtectVirtualMemory(
                self.0,
                &mut temp_address,
                &mut region_size,
                protect,
                &mut old_protect,
            )
        };
        
        if !NT_SUCCESS(status) {
            return Err(status);
        }
        
        Ok(ProcessMemoryProtectorSession {
            handle: self.0,
            address: page_address,
            size: region_size,
            old_protect,
        })
    }
}
