use ntapi::ntmmapi::*;
use ntapi::ntrtl::*;
use utils::NtDll;
use winapi::shared::basetsd::SIZE_T;
use winapi::shared::ntdef::{PVOID, UNICODE_STRING};
use winapi::shared::ntstatus::STATUS_SUCCESS;
use winapi::um::winnt::PAGE_EXECUTE_READWRITE;
use winapi::um::winnt::{HANDLE, MEM_COMMIT, MEM_RESERVE, PAGE_READWRITE};

use crate::types::*;

pub fn get_veh_list_struct() -> *mut VECTORED_HANDLER_LIST {
    unsafe {
        let ntdll = NtDll::from_current_process();
        let ptr_to_struct = ntdll.vectored_handler_list() as *mut VECTORED_HANDLER_LIST;

        ptr_to_struct
    }
}

pub fn read_memory_at_address(process_handle: HANDLE, address: usize, size: usize) -> Result<Vec<u8>, i32> {
    unsafe {
        let mut buffer = vec![0u8; size];
        let mut bytes_read: usize = 0;
        
        let status = NtReadVirtualMemory(
            process_handle,
            address as _,
            buffer.as_mut_ptr() as *mut _,
            size,
            &mut bytes_read,
        );
        
        if status >= 0 {
            buffer.truncate(bytes_read);
            Ok(buffer)
        } else {
            Err(status)
        }
    }
}

pub fn protect_memory_at_address(
    process_handle: HANDLE, 
    address: usize, 
    size: usize, 
    new_protect: u32
) -> Result<u32, i32> {
    unsafe {
        let mut base_address = address as *mut _;
        let mut region_size = size;
        let mut old_protect: u32 = 0;
        
        let status = NtProtectVirtualMemory(
            process_handle,
            &mut base_address,
            &mut region_size,
            new_protect,
            &mut old_protect,
        );
        
        if status == STATUS_SUCCESS {
            Ok(old_protect)
        } else {
            Err(status)
        }
    }
}

pub fn alloc_memory_at_address(
    process_handle: HANDLE, 
    address: Option<usize>, 
    size: usize,
    protect: u32
) -> Result<usize, i32> {
    unsafe {
        let mut base_address = match address {
            Some(addr) => addr as *mut _,
            None => core::ptr::null_mut(),
        };
        let mut region_size = size;
        
        let status = NtAllocateVirtualMemory(
            process_handle,
            &mut base_address,
            0, // ZeroBits - let system decide alignment
            &mut region_size,
            MEM_COMMIT | MEM_RESERVE,
            protect,
        );
        
        if status == STATUS_SUCCESS {
            Ok(base_address as usize)
        } else {
            Err(status)
        }
    }
}

pub fn write_value_to_address<T>(process_handle: HANDLE, address: usize, value: &T) -> Result<(), i32> {
    unsafe {
        let bytes = core::slice::from_raw_parts(
            value as *const T as *const u8,
            core::mem::size_of::<T>()
        );
        write_memory_to_address(process_handle, address, bytes)
    }
}

pub fn write_memory_to_address(process_handle: HANDLE, address: usize, buffer: &[u8]) -> Result<(), i32> {
    unsafe {
        let mut bytes_read: usize = 0;
        
        let status = NtWriteVirtualMemory(
            process_handle,
            address as _,
            buffer.as_ptr() as *mut _,
            buffer.len() as _,
            &mut bytes_read,
        );
        
        if status >= 0 {
            Ok(())
        } else {
            Err(status)
        }
    }
}

pub fn get_mapped_file_name(handle: HANDLE, base_address: PVOID) -> Option<String> {
    unsafe {
        let mut buffer = [0u8; 1024];
        let mut bytes_read: SIZE_T = 0;
        
        let status = NtQueryVirtualMemory(
            handle,
            base_address,
            MemoryMappedFilenameInformation,
            buffer.as_mut_ptr() as PVOID,
            buffer.len() as SIZE_T,
            &mut bytes_read,
        );
        
        if status < 0 {
            return None;
        }
        
        if bytes_read >= core::mem::size_of::<UNICODE_STRING>() as SIZE_T {
            let us = &*(buffer.as_ptr() as *const UNICODE_STRING);
            let len = (us.Length as usize) / 2;
            
            if len > 0 {
                let slice = core::slice::from_raw_parts(us.Buffer, len);
                return Some(String::from_utf16_lossy(slice));
            }
        }
        
        None
    }
}