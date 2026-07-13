
use ntapi::{ntpsapi::NtCurrentProcess};
use winapi::{ctypes::c_void, shared::ntdef::{NT_SUCCESS, NTSTATUS, PVOID}};

use crate::{println, syscalls::NtWriteVirtualMemory};

pub struct ProcessMemoryWriter;

impl ProcessMemoryWriter {
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

    pub fn write(address: PVOID, buffer: &mut [u8]) -> Result<(), NTSTATUS> {
        let handle = NtCurrentProcess;
        Self::write_remote(handle, address, buffer)
    }
}
