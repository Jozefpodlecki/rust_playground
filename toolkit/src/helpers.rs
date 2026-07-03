use core::fmt::{self, Display, Formatter};

use ntapi::{ntexapi::NtDelayExecution, ntmmapi::{MemoryBasicInformation, MemoryMappedFilenameInformation, NtProtectVirtualMemory, NtQueryVirtualMemory, NtReadVirtualMemory, NtWriteVirtualMemory}, ntpebteb::PEB, ntpsapi::NtCurrentProcess, ntrtl::{HEAP_INFORMATION, RTL_USER_PROCESS_PARAMETERS}};
use winapi::{ctypes::c_void, shared::ntdef::{HANDLE, LIST_ENTRY, NT_SUCCESS, NTSTATUS, PVOID, UNICODE_STRING}, um::winnt::{LARGE_INTEGER, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE_READWRITE, RTL_RUN_ONCE}};

use crate::{MemoryRegionIterator, U16CStackString, print, println, types::{ByteBlock, HEAP}};




#[unsafe(naked)]
pub unsafe fn get_peb() -> *mut PEB {
    core::arch::naked_asm!(
        "mov rax, gs:[0x60]",
        "ret"
    );
}

pub struct ProcessEnvironmentBlock(*mut PEB);

impl ProcessEnvironmentBlock {
    pub fn current_process() -> Self {
        let peb: *mut PEB;
        unsafe {
            core::arch::asm!(
                "mov {0}, gs:[0x60]",
                out(reg) peb,
                options(nostack, readonly)
            );
        }
        Self(peb)
    }

    pub fn process_params(&self) -> *mut RTL_USER_PROCESS_PARAMETERS {
        unsafe { (*self.0).ProcessParameters }
    }
    
    pub fn image_base(&self) -> *mut c_void {
        unsafe { (*self.0).ImageBaseAddress }
    }

    pub fn process_heap(&self) -> *mut HEAP {
        unsafe {
            (*self.0).ProcessHeap as *mut HEAP
            // let raw_heap = (*self.0).ProcessHeap;
            // // The _HEAP structure starts at offset 0x10 (after the initial HEAP_ENTRY)
            // (raw_heap as usize + 0x10) as *mut HEAP
        }
    }

    pub fn executable_path(&self) -> U16CStackString<260> {
        let params = unsafe { &*(*self.0).ProcessParameters };
        let image_path: UNICODE_STRING = params.ImagePathName;
        U16CStackString::<260>::from_ptr(image_path.Buffer).unwrap()
    }

}

pub struct ProcessMemoryBytesReader;

impl ProcessMemoryBytesReader {
    pub fn read_remote<const N: usize>(handle: *mut c_void, address: PVOID) -> Result<ByteBlock<N>, NTSTATUS> {
        let mut buffer = ByteBlock::<N>::new();
        let mut bytes_read: usize = 0;
        
        let status = unsafe {
            NtReadVirtualMemory(
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

    pub fn read<const N: usize>(address: PVOID) -> Result<ByteBlock<N>, NTSTATUS> {
        let handle = NtCurrentProcess;
        let mut buffer = ByteBlock::<N>::new();
        let mut bytes_read: usize = 0;
        
        let status = unsafe {
            NtReadVirtualMemory(
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