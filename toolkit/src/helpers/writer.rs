
use ntapi::{ntmmapi::NtWriteVirtualMemory, ntpsapi::NtCurrentProcess};
use winapi::{ctypes::c_void, shared::ntdef::{NT_SUCCESS, NTSTATUS, PVOID}};

pub struct ProcessMemoryBytesWriter;

impl ProcessMemoryBytesWriter {
    pub fn write_remote(handle: *mut c_void ,address: PVOID, buffer: &mut [u8]) -> Result<(), NTSTATUS> {
        let mut bytes_read: usize = 0;
        let status = unsafe {
            NtWriteVirtualMemory(
                handle,
                address,
                buffer.as_mut_ptr() as _,
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
