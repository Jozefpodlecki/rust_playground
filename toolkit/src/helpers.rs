use ntapi::{ntexapi::NtDelayExecution, ntmmapi::{NtProtectVirtualMemory, NtReadVirtualMemory, NtWriteVirtualMemory}, ntpebteb::PEB, ntpsapi::NtCurrentProcess};
use winapi::{shared::ntdef::{HANDLE, NT_SUCCESS, NTSTATUS, PVOID}, um::winnt::{LARGE_INTEGER, PAGE_EXECUTE_READWRITE}};

use crate::MemoryRegionIterator;

#[unsafe(naked)]
pub unsafe fn get_peb() -> *mut PEB {
    core::arch::naked_asm!(
        "mov rax, gs:[0x60]",
        "ret"
    );
}

pub struct ProcessMemoryBytesReader;

impl ProcessMemoryBytesReader {
    pub fn read<const N: usize>(address: PVOID) -> Result<[u8; N], NTSTATUS> {
        let handle = NtCurrentProcess;
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

pub struct ProcessMemoryBytesWriter;

impl ProcessMemoryBytesWriter {
    pub fn write(address: PVOID, buffer: &mut [u8]) -> Result<(), NTSTATUS> {
        let handle = NtCurrentProcess;
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
}

pub struct ProcessMemoryProtector {
    address: PVOID,
    size: usize,
    old_protect: u32,
}

impl ProcessMemoryProtector {
    pub fn make_writable(address: PVOID, size: usize) -> Result<Self, NTSTATUS> {
        let page_size = 0x1000;
        let page_address = (address as usize & !(page_size - 1)) as PVOID;
        let region_size = ((address as usize + size - 1) & !(page_size - 1)) + page_size - (address as usize & !(page_size - 1));
        let mut region_size = region_size;
        let mut old_protect: u32 = 0;
        let mut temp_address = page_address;
        
        let status = unsafe {
            NtProtectVirtualMemory(
                NtCurrentProcess,
                &mut temp_address,
                &mut region_size,
                PAGE_EXECUTE_READWRITE,
                &mut old_protect,
            )
        };
        
        if !NT_SUCCESS(status) {
            return Err(status);
        }
        
        Ok(Self {
            address: page_address,
            size: region_size,
            old_protect,
        })
    }
    
    pub fn make_writable_with(address: PVOID, size: usize, protect: u32) -> Result<Self, NTSTATUS> {
        let page_size = 0x1000;
        let page_address = (address as usize & !(page_size - 1)) as PVOID;
        let region_size = ((address as usize + size - 1) & !(page_size - 1)) + page_size - (address as usize & !(page_size - 1));
        let mut region_size = region_size;
        let mut old_protect: u32 = 0;
        let mut temp_address = page_address;
        
        let status = unsafe {
            NtProtectVirtualMemory(
                NtCurrentProcess,
                &mut temp_address,
                &mut region_size,
                protect,
                &mut old_protect,
            )
        };
        
        if !NT_SUCCESS(status) {
            return Err(status);
        }
        
        Ok(Self {
            address: page_address,
            size: region_size,
            old_protect,
        })
    }
    
    /// Restores the original page protection
    pub fn restore(&mut self) -> Result<(), NTSTATUS> {
        let mut address = self.address;
        let mut size = self.size;
        let mut new_protect: u32 = 0;
        
        let status = unsafe {
            NtProtectVirtualMemory(
                NtCurrentProcess,
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