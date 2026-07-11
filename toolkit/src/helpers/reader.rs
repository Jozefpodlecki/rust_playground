use core::fmt::{self, Display, Formatter};

use ntapi::{ntexapi::NtDelayExecution, ntmmapi::{MemoryBasicInformation}, ntpebteb::PEB, ntpsapi::NtCurrentProcess, ntrtl::{HEAP_INFORMATION, RTL_USER_PROCESS_PARAMETERS}};
use winapi::{ctypes::c_void, shared::ntdef::{HANDLE, LIST_ENTRY, NT_SUCCESS, NTSTATUS, PVOID, UNICODE_STRING}, um::winnt::{LARGE_INTEGER, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE_READWRITE, RTL_RUN_ONCE}};

use crate::{MemoryRegionIterator, U16CStackString, print, println, types::{ByteBlock, HEAP}};

pub struct ProcessMemoryReader;

impl ProcessMemoryReader {
    pub fn read_remote_bytes_fixed<const N: usize>(handle: *mut c_void, address: PVOID) -> Result<ByteBlock<N>, NTSTATUS> {
        let mut buffer = ByteBlock::<N>::new();
        let mut bytes_read: usize = 0;
        
        let status = unsafe {
            crate::syscalls::NtReadVirtualMemory(
                handle,
                address,
                buffer.as_mut_bytes().as_mut_ptr() as _,
                buffer.len(),
                &mut bytes_read,
            )
        };

        if NT_SUCCESS(status) {
            Ok(buffer)
        } else {
            Err(status)
        }
    }

    pub fn read_remote<T: Sized>(handle: *mut c_void, address: PVOID) -> Result<T, NTSTATUS> {
        let mut buffer = core::mem::MaybeUninit::<T>::uninit();
        let mut bytes_read: usize = 0;
        
        let status = unsafe {
            crate::syscalls::NtReadVirtualMemory(
                handle,
                address,
                buffer.as_mut_ptr() as *mut _,
                core::mem::size_of::<T>(),
                &mut bytes_read,
            )
        };

        if NT_SUCCESS(status) {
            Ok(unsafe { buffer.assume_init() })
        } else {
            Err(status)
        }
    }

    pub fn read_bytes_fixed<const N: usize>(address: PVOID) -> Result<ByteBlock<N>, NTSTATUS> {
        let handle = NtCurrentProcess;
        let mut buffer = ByteBlock::<N>::new();
        let mut bytes_read: usize = 0;
        
        let status = unsafe {
            crate::syscalls::NtReadVirtualMemory(
                handle,
                address,
                buffer.as_mut_bytes().as_mut_ptr() as _,
                buffer.len(),
                &mut bytes_read,
            )
        };

        if NT_SUCCESS(status) {
            Ok(buffer)
        } else {
            Err(status)
        }
    }
}
