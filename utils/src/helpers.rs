use ntapi::{ntexapi::NtDelayExecution, ntmmapi::NtReadVirtualMemory};
use winapi::{shared::ntdef::{HANDLE, NT_SUCCESS, NTSTATUS, PVOID}, um::winnt::LARGE_INTEGER};

pub struct ProcessMemoryBytesReader;

impl ProcessMemoryBytesReader {
    pub fn read<const N: usize>(address: PVOID) -> Result<[u8; N], NTSTATUS> {
        let handle = -1isize as HANDLE;
        let mut buffer = [0u8; N];
        let mut bytes_read: usize = 0;
        let status = unsafe {
            NtReadVirtualMemory(
                handle,
                address,
                buffer.as_mut_ptr() as _,
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