use core::{fmt, slice};

use ntapi::{ntexapi::{SYSTEM_BASIC_INFORMATION, SystemBasicInformation}, ntmmapi::{MEMORY_INFORMATION_CLASS, MemoryBasicInformation, MemoryMappedFilenameInformation}};
use winapi::{shared::{basetsd::SIZE_T, ntdef::UNICODE_STRING}, um::winnt::*};

use crate::{NtError, syscalls::{NtQuerySystemInformation, NtQueryVirtualMemory}};

const MAX_MODULE_NAME_LEN: usize = 260;

pub struct MemoryInformation {
    handle: HANDLE,
    inner: MEMORY_BASIC_INFORMATION
}

pub struct MemoryMappedFilename {
    buf: [u16; MAX_MODULE_NAME_LEN],
    len: usize
}

impl MemoryMappedFilename {
    pub const fn new() -> Self {
        Self { buf: [0; MAX_MODULE_NAME_LEN], len: 0 }
    }

    pub const fn from_units(units: &[u16]) -> Self {
        let mut this = Self::new();
        let n = units.len().min(MAX_MODULE_NAME_LEN);
        this.buf[..n].copy_from_slice(&units[..n]);
        this.len = n;
        this
    }

    pub const fn as_slice(&self) -> &[u16] {
        unsafe { core::slice::from_raw_parts(self.buf.as_ptr(), self.len) }
    }

    pub const fn len(&self) -> usize { self.len }
    pub const fn is_empty(&self) -> bool { self.len == 0 }
}

impl fmt::Display for MemoryMappedFilename {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use fmt::Write;
        for c in char::decode_utf16(self.as_slice().iter().copied()) {
            match c {
                Ok(c) => f.write_char(c)?,
                Err(_) => f.write_char(char::REPLACEMENT_CHARACTER)?,
            }
        }
        Ok(())
    }
}

impl MemoryInformation {
    pub const fn new(handle: HANDLE, inner: MEMORY_BASIC_INFORMATION) -> Self {
        
        Self {
            handle,
            inner,
        }
    }
    
    pub fn get_mapped_file_name(&self) -> Result<MemoryMappedFilename, NtError> {
        let mut buffer = [0u8; MAX_MODULE_NAME_LEN];
        let mut bytes_read: SIZE_T = 0;

        NtQueryVirtualMemory(
            self.handle as _,
            self.base_address() as _,
            MemoryMappedFilenameInformation,
            buffer.as_mut_ptr() as _,
            buffer.len() as SIZE_T,
            &mut bytes_read,
        ).ok()?;

        let us = unsafe { &*(buffer.as_ptr() as *const UNICODE_STRING) };

        let len_units = us.Length as usize / 2;
        if len_units == 0 || len_units > MAX_MODULE_NAME_LEN {
            return Ok(MemoryMappedFilename::new());
        }

        let units = unsafe { slice::from_raw_parts(us.Buffer, len_units) };

        Ok(MemoryMappedFilename::from_units(units))
    }
    
    pub fn base_address(&self) -> usize {
        self.inner.BaseAddress as usize
    }
    
    pub fn allocation_base(&self) -> PVOID {
        self.inner.AllocationBase
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
        writeln!(f, "  Allocation Base:    {:p}", self.allocation_base())?;
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

    pub fn from_address(handle: HANDLE, start_address: PVOID) -> Self {
        Self {
            handle,
            address: start_address,
        }
    }
}

impl Iterator for MemoryRegionIterator {
    type Item = MemoryInformation;
    
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let mut mbi: MEMORY_BASIC_INFORMATION = core::mem::zeroed();
            let mut return_length: SIZE_T = 0;
            
            NtQueryVirtualMemory(
                self.handle as _,
                self.address as _,
                MemoryBasicInformation,
                &mut mbi as *mut _ as _,
                core::mem::size_of::<MEMORY_BASIC_INFORMATION>() as _,
                &mut return_length,
            ).ok().ok()?;
            
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

pub struct MemoryRegionReverseIterator {
    handle: HANDLE,
    address: PVOID,
    max_address: usize,
    initialized: bool,
}

impl MemoryRegionReverseIterator {
    pub fn new(handle: HANDLE, start_address: PVOID) -> Self {
        let max_address = if start_address.is_null() {
            0x7FFFFFFEFFFFusize
        } else {
            start_address as usize
        };
        
        Self {
            handle,
            address: max_address as PVOID,
            max_address,
            initialized: false,
        }
    }

    pub fn from_max_address(handle: HANDLE) -> Self {
        Self::new(handle, core::ptr::null_mut())
    }

    fn query_region_at(&self, address: PVOID) -> Option<MemoryInformation> {
        unsafe {
            let mut mbi: MEMORY_BASIC_INFORMATION = core::mem::zeroed();
            let mut return_length: SIZE_T = 0;
            
            NtQueryVirtualMemory(
                self.handle as _,
                address as _,
                MemoryBasicInformation,
                &mut mbi as *mut _ as _,
                core::mem::size_of::<MEMORY_BASIC_INFORMATION>() as SIZE_T,
                &mut return_length,
            ).ok().ok()?;
            
            if mbi.RegionSize == 0 {
                return None;
            }
            
            Some(MemoryInformation::new(self.handle, mbi))
        }
    }
}

impl Iterator for MemoryRegionReverseIterator {
    type Item = MemoryInformation;
    
    fn next(&mut self) -> Option<Self::Item> {
        let address = self.address as usize;

        if address == 0 {
            return None;
        }

        let info = self.query_region_at(address as PVOID)?;

        let base = info.base_address();

        if base == 0 {
            self.address = core::ptr::null_mut();
        } else {
            self.address = info.base_address().saturating_sub(1) as _;
        }

        Some(info)
    }
}

impl MemoryRegionIterator {
    pub fn rev(self) -> MemoryRegionReverseIterator {
        let mut iter = self;
        let mut last_address = 0usize;
        
        for region in iter.by_ref() {
            let end = region.range_end();
            if end > last_address {
                last_address = end;
            }
        }
        
        if last_address == 0 {
            MemoryRegionReverseIterator::new(iter.handle, core::ptr::null_mut())
        } else {
            MemoryRegionReverseIterator::new(iter.handle, (last_address - 1) as PVOID)
        }
    }
}

pub fn maximum_user_address() -> usize {
    let mut info: SYSTEM_BASIC_INFORMATION = unsafe { core::mem::zeroed() };

    let status = NtQuerySystemInformation(
        SystemBasicInformation,
        &mut info as *mut _ as _,
        core::mem::size_of::<SYSTEM_BASIC_INFORMATION>() as u32,
        core::ptr::null_mut(),
    );

    info.MaximumUserModeAddress as usize
}