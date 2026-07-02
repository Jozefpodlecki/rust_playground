use core::fmt::{self, Display, Formatter};

use ntapi::{ntexapi::NtDelayExecution, ntmmapi::{NtProtectVirtualMemory, NtReadVirtualMemory, NtWriteVirtualMemory}, ntpebteb::PEB, ntpsapi::NtCurrentProcess, ntrtl::{HEAP_INFORMATION, RTL_USER_PROCESS_PARAMETERS}};
use winapi::{shared::ntdef::{HANDLE, LIST_ENTRY, NT_SUCCESS, NTSTATUS, PVOID, UNICODE_STRING}, um::winnt::{LARGE_INTEGER, PAGE_EXECUTE_READWRITE, RTL_RUN_ONCE}};

use crate::{HEAP, MemoryRegionIterator, U16CStackString};




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

#[repr(transparent)]
pub struct ProcessMemoryBytes<const N: usize>([u8; N]);

impl<const N: usize> ProcessMemoryBytes<N> {
    pub fn new() -> Self {
        Self([0u8; N])
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    
    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        &mut self.0
    }
    
    pub fn len(&self) -> usize {
        N
    }
    
    pub fn is_empty(&self) -> bool {
        N == 0
    }
    
    pub fn get(&self, index: usize) -> Option<u8> {
        if index < N {
            Some(self.0[index])
        } else {
            None
        }
    }
}

impl<const N: usize> Default for ProcessMemoryBytes<N> {
    fn default() -> Self {
        Self([0u8; N])
    }
}

impl<const N: usize> Display for ProcessMemoryBytes<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "ProcessMemoryBytes ({} bytes):", N)?;
        
        for (i, chunk) in self.0.chunks(16).enumerate() {
            // Hex part
            write!(f, "{:04X}: ", i * 16)?;
            
            for byte in chunk.iter() {
                write!(f, "{:02X} ", byte)?;
            }
            
            let padding = 16 - chunk.len();
            for _ in 0..padding {
                write!(f, "   ")?;
            }
            
            write!(f, " |")?;
            for byte in chunk.iter() {
                let c = *byte;
                if c >= 0x20 && c <= 0x7E {
                    write!(f, "{}", c as char)?;
                } else {
                    write!(f, ".")?;
                }
            }
            for _ in 0..padding {
                write!(f, " ")?;
            }
            writeln!(f, "|")?;
        }
        
        Ok(())
    }
}

impl<const N: usize> AsRef<[u8]> for ProcessMemoryBytes<N> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<const N: usize> AsMut<[u8]> for ProcessMemoryBytes<N> {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl<const N: usize> core::ops::Deref for ProcessMemoryBytes<N> {
    type Target = [u8; N];
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> core::ops::DerefMut for ProcessMemoryBytes<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct ProcessMemoryBytesReader;

impl ProcessMemoryBytesReader {
    pub fn read<const N: usize>(address: PVOID) -> Result<ProcessMemoryBytes<N>, NTSTATUS> {
        let handle = NtCurrentProcess;
        let mut buffer = ProcessMemoryBytes::<N>::new();
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