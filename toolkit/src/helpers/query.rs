use core::fmt::{self, Display, Formatter};

use ntapi::{ntmmapi::{MemoryBasicInformation, MemoryMappedFilenameInformation}, ntpebteb::PEB, ntpsapi::NtCurrentProcess, ntrtl::{HEAP_INFORMATION, RTL_USER_PROCESS_PARAMETERS}};
use winapi::{ctypes::c_void, shared::ntdef::{HANDLE, LIST_ENTRY, NT_SUCCESS, NTSTATUS, PVOID, UNICODE_STRING}, um::winnt::{LARGE_INTEGER, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE_READWRITE, RTL_RUN_ONCE}};

use crate::{MemoryRegionIterator, U16CStackString, print, println, syscalls::{NtDelayExecution, NtQueryVirtualMemory}, types::{ByteBlock, HEAP}};

pub struct Sleeper;

impl Sleeper {
    pub fn sleep(milliseconds: u32) {
        let mut delay: LARGE_INTEGER = unsafe { core::mem::zeroed() };
        unsafe {
            *delay.QuadPart_mut() = -(milliseconds as i64) * 10_000;
            NtDelayExecution(0, &mut delay);
        }
    }
}

pub struct ProcessMemoryQuery;

impl ProcessMemoryQuery {
    pub fn query_basic(address: PVOID) -> Result<MEMORY_BASIC_INFORMATION, NTSTATUS> {
        let mut mbi: MEMORY_BASIC_INFORMATION = unsafe { core::mem::zeroed() };
        let mut return_length: usize = 0;
        
        let status = unsafe {
            NtQueryVirtualMemory(
                NtCurrentProcess,
                address,
                MemoryBasicInformation,
                &mut mbi as *mut _ as PVOID,
                core::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                &mut return_length,
            )
        };
        
        if NT_SUCCESS(status) {
            Ok(mbi)
        } else {
            Err(status)
        }
    }

    pub fn query_mapped_filename(address: PVOID) -> Result<U16CStackString<260>, NTSTATUS> {
        let mut return_length: usize = 0;
        let mut buffer: [u16; 260] = [0; 260];

        let status = unsafe {
            NtQueryVirtualMemory(
                NtCurrentProcess,
                address,
                MemoryMappedFilenameInformation as u32,
                buffer.as_mut_ptr() as PVOID,
                buffer.len() * 2,
                &mut return_length,
            )
        };

        if NT_SUCCESS(status) && return_length > 0 {
            let unicode_string = unsafe {
                let ptr = buffer.as_ptr() as *const UNICODE_STRING;
                &*ptr
            };
            
            unsafe { Ok(U16CStackString::from_raw_parts(unicode_string.Buffer as _, unicode_string.Length as _).unwrap()) }
        } else {
            Err(status)
        }
    }
}