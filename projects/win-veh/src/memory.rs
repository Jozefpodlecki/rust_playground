use core::{fmt, slice};

use ntapi::ntmmapi::{MEMORY_INFORMATION_CLASS, MemoryBasicInformation, MemoryMappedFilenameInformation, NtQueryVirtualMemory};
use winapi::{shared::{basetsd::SIZE_T, ntdef::UNICODE_STRING}, um::winnt::*};

const MAX_MODULE_NAME_LEN: usize = 260;

pub struct MemoryInformation {
    inner: MEMORY_BASIC_INFORMATION,
    mapped_name: Option<String>,
}

impl MemoryInformation {
      pub fn new(handle: HANDLE, info: MEMORY_BASIC_INFORMATION) -> Self {
        let mapped_name = if info.Type == MEM_IMAGE as u32 {
            Self::get_mapped_file_name(handle, info.BaseAddress)
        } else {
            None
        };
        
        Self {
            inner: info,
            mapped_name,
        }
    }

    pub fn mapped_file_name(&self) -> Option<&str> {
        self.mapped_name.as_deref()
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
                    let slice = slice::from_raw_parts(us.Buffer, len);
                    return Some(String::from_utf16_lossy(slice));
                }
            }
            
            None
        }
    }
    
    pub fn base_address(&self) -> usize {
        self.inner.BaseAddress as usize
    }
    
    pub fn allocation_base(&self) -> usize {
        self.inner.AllocationBase as usize
    }
    
    pub fn allocation_protect(&self) -> u32 {
        self.inner.AllocationProtect
    }
    
    pub fn region_size(&self) -> usize {
        self.inner.RegionSize
    }
    
    pub fn state(&self) -> u32 {
        self.inner.State
    }
    
    pub fn protect(&self) -> u32 {
        self.inner.Protect
    }
    
    pub fn type_(&self) -> u32 {
        self.inner.Type
    }
    
    pub fn is_committed(&self) -> bool {
        self.inner.State == MEM_COMMIT
    }
    
    pub fn is_reserved(&self) -> bool {
        self.inner.State == MEM_RESERVE
    }
    
    pub fn is_free(&self) -> bool {
        self.inner.State == MEM_FREE
    }
    
    pub fn is_private(&self) -> bool {
        self.inner.Type == MEM_PRIVATE
    }
    
    pub fn is_mapped(&self) -> bool {
        self.inner.Type == MEM_MAPPED
    }
    
    pub fn is_image(&self) -> bool {
        self.inner.Type == MEM_IMAGE as u32
    }
    
    pub fn is_readable(&self) -> bool {
        let protect = self.inner.Protect;
        protect & PAGE_NOACCESS == 0
            && protect & PAGE_GUARD == 0
            && (protect & (PAGE_READONLY | PAGE_READWRITE | PAGE_WRITECOPY | PAGE_EXECUTE_READ | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY)) != 0
    }
    
    pub fn is_writable(&self) -> bool {
        let protect = self.inner.Protect;
        protect & (PAGE_READWRITE | PAGE_WRITECOPY | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY) != 0
    }
    
    pub fn is_executable(&self) -> bool {
        let protect = self.inner.Protect;
        protect & (PAGE_EXECUTE | PAGE_EXECUTE_READ | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY) != 0
    }
    
    pub fn is_guard(&self) -> bool {
        self.inner.Protect & PAGE_GUARD != 0
    }
    
    pub fn is_no_cache(&self) -> bool {
        self.inner.Protect & PAGE_NOCACHE != 0
    }
    
    pub fn is_write_combine(&self) -> bool {
        self.inner.Protect & PAGE_WRITECOMBINE != 0
    }

    pub fn range_start(&self) -> usize {
        self.base_address()
    }
    
    pub fn range_end(&self) -> usize {
        self.base_address().saturating_add(self.region_size())
    }
    
    pub fn range(&self) -> (usize, usize) {
        (self.range_start(), self.range_end())
    }

    fn allocation_protect_str(&self) -> &'static str {
        match self.inner.AllocationProtect {
            PAGE_NOACCESS => "PAGE_NOACCESS",
            PAGE_READONLY => "PAGE_READONLY",
            PAGE_READWRITE => "PAGE_READWRITE",
            PAGE_WRITECOPY => "PAGE_WRITECOPY",
            PAGE_EXECUTE => "PAGE_EXECUTE",
            PAGE_EXECUTE_READ => "PAGE_EXECUTE_READ",
            PAGE_EXECUTE_READWRITE => "PAGE_EXECUTE_READWRITE",
            PAGE_EXECUTE_WRITECOPY => "PAGE_EXECUTE_WRITECOPY",
            _ => "UNKNOWN",
        }
    }
    
    fn state_str(&self) -> &'static str {
        match self.inner.State {
            MEM_COMMIT => "COMMIT",
            MEM_RESERVE => "RESERVE",
            MEM_FREE => "FREE",
            MEM_DECOMMIT => "DECOMMIT",
            MEM_RELEASE => "RELEASE",
            _ => "UNKNOWN",
        }
    }
    
    fn type_str(&self) -> &'static str {
        match self.inner.Type {
            MEM_PRIVATE => "PRIVATE",
            MEM_MAPPED => "MAPPED",
            t if t == MEM_IMAGE as u32 => "IMAGE",
            _ => "UNKNOWN",
        }
    }
}

impl fmt::Display for MemoryInformation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Memory Region:")?;
        writeln!(f, "  Range:                {:#X} - {:#X} (size: {:#X} bytes)", 
            self.range_start(), 
            self.range_end(), 
            self.region_size()
        )?;
        writeln!(f, "  Base Address:      {:#X}", self.base_address())?;
        writeln!(f, "  Allocation Base:    {:#X}", self.allocation_base())?;
        writeln!(f, "  Region Size:        {:#X} ({} bytes)", self.region_size(), self.region_size())?;
        writeln!(f, "  State:              {} ({:#X})", self.state_str(), self.state())?;
        writeln!(f, "  Type:               {} ({:#X})", self.type_str(), self.type_())?;
        writeln!(f, "  Allocation Protect:   {} ({:#X})", self.allocation_protect_str(), self.allocation_protect())?;
        
        write!(f, "  Protect:            ")?;
        
        let p = self.inner.Protect;
        let mut first = true;
        
        macro_rules! write_flag {
            ($flag:expr, $name:literal) => {
                if p & $flag != 0 {
                    if !first { write!(f, " | ")?; }
                    write!(f, "{}", $name)?;
                    first = false;
                }
            };
        }
        
        write_flag!(PAGE_NOACCESS, "NOACCESS");
        write_flag!(PAGE_READONLY, "READONLY");
        write_flag!(PAGE_READWRITE, "READWRITE");
        write_flag!(PAGE_WRITECOPY, "WRITECOPY");
        write_flag!(PAGE_EXECUTE, "EXECUTE");
        write_flag!(PAGE_EXECUTE_READ, "EXECUTE_READ");
        write_flag!(PAGE_EXECUTE_READWRITE, "EXECUTE_READWRITE");
        write_flag!(PAGE_EXECUTE_WRITECOPY, "EXECUTE_WRITECOPY");
        write_flag!(PAGE_GUARD, "GUARD");
        write_flag!(PAGE_NOCACHE, "NOCACHE");
        write_flag!(PAGE_WRITECOMBINE, "WRITECOMBINE");
        
        if first { write!(f, "0")?; }
        writeln!(f, " ({:#010X})", p)?;
        
        write!(f, "  Flags:")?;
        if self.is_committed() { write!(f, " COMMITTED")?; }
        if self.is_readable() { write!(f, " READABLE")?; }
        if self.is_writable() { write!(f, " WRITABLE")?; }
        if self.is_executable() { write!(f, " EXECUTABLE")?; }
        if self.is_guard() { write!(f, " GUARD")?; }
        if self.is_no_cache() { write!(f, " NO_CACHE")?; }
        if self.is_write_combine() { write!(f, " WRITE_COMBINE")?; }
        
        Ok(())
    }
}

pub struct MemoryRegionIterator {
    handle: HANDLE,
    address: PVOID,
}

impl MemoryRegionIterator {
    pub fn new(handle: HANDLE) -> Self {
        Self {
            handle,
            address: core::ptr::null_mut(),
        }
    }
}

impl Iterator for MemoryRegionIterator {
    type Item = MemoryInformation;
    
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let mut mbi: MEMORY_BASIC_INFORMATION = core::mem::zeroed();
            let mut return_length: SIZE_T = 0;
            
            let status = NtQueryVirtualMemory(
                self.handle,
                self.address,
                MemoryBasicInformation,
                &mut mbi as *mut _ as PVOID,
                core::mem::size_of::<MEMORY_BASIC_INFORMATION>() as SIZE_T,
                &mut return_length,
            );
            
            if status < 0 {
                return None;
            }
            
            let base = mbi.BaseAddress as usize;
            let size = mbi.RegionSize;
            
            if size == 0 {
                return None;
            }
            
            self.address = (base as usize).saturating_add(size) as PVOID;
            Some(MemoryInformation::new(self.handle, mbi))
        }
    }
}