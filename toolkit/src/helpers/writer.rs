
use core::mem;

use ntapi::{ntpsapi::NtCurrentProcess};
use winapi::{ctypes::c_void, shared::ntdef::{NT_SUCCESS, NTSTATUS, PVOID}};

use crate::{println, syscalls::NtWriteVirtualMemory};

pub struct ProcessMemoryWriter;

impl ProcessMemoryWriter {
    pub fn write_struct<T>(address: PVOID, value: &T) -> Result<(), NTSTATUS> {
        let size = mem::size_of::<T>();
        let slice = unsafe {
            core::slice::from_raw_parts(
                value as *const T as *const u8,
                size,
            )
        };
        Self::write(address, slice)
    }

    pub fn write_remote(handle: *mut c_void, address: PVOID, buffer: &[u8]) -> Result<(), NTSTATUS> {
        let mut bytes_read: usize = 0;
        let status = unsafe {
            NtWriteVirtualMemory(
                handle,
                address,
                buffer.as_ptr() as *mut _,
                buffer.len(),
                &mut bytes_read,
            )
        };

        if NT_SUCCESS(status) {
            Ok(())
        } else {
            Err(status)
        }
    }

    pub fn write_struct_remote<T>(
        handle: *mut c_void,
        address: PVOID,
        value: &T,
    ) -> Result<(), NTSTATUS> {
        let size = mem::size_of::<T>();
        let slice = unsafe {
            core::slice::from_raw_parts(
                value as *const T as *const u8,
                size,
            )
        };
        Self::write_remote(handle, address, slice)
    }

    pub fn write(address: PVOID, buffer: &[u8]) -> Result<(), NTSTATUS> {
        let handle = NtCurrentProcess;
        Self::write_remote(handle, address, buffer)
    }
}
