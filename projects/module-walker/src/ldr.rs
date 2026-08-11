use core::{fmt, slice};

use ntapi::ntldr::LDR_DATA_TABLE_ENTRY;
use toolkit::ProcessEnvironmentBlock;
use winapi::shared::ntdef::LIST_ENTRY;

use crate::exports::ModuleExportsIter;

pub struct LdrModuleEntry {
    pub base: ModuleBase,
    pub size: ModuleSize,
    pub file_name: ModuleName,
}

impl LdrModuleEntry {
    pub fn new(entry: &LDR_DATA_TABLE_ENTRY) -> Self {
        let file_name = if !entry.BaseDllName.Buffer.is_null() {
            let len = (entry.BaseDllName.Length / 2) as usize;
            let u16_slice = unsafe { core::slice::from_raw_parts(entry.BaseDllName.Buffer, len) };
            ModuleName::new(u16_slice)
        } else {
            ModuleName::new(&[])
        };
        
        Self {
            base: ModuleBase(entry.DllBase as usize),
            size: ModuleSize(entry.SizeOfImage as usize),
            file_name,
        }
    }

    pub fn exports(&self) -> Option<ModuleExportsIter> {
        ModuleExportsIter::new(self.base.0 as _)
    }
    
    pub fn base_address(&self) -> usize {
        self.base.0
    }
    
    pub fn size_of_image(&self) -> usize {
        self.size.0
    }
    
    pub fn file_name_str(&self) -> Option<&str> {
        self.file_name.as_str()
    }
}

pub struct LdrModuleIterator {
    current: *mut LIST_ENTRY,
    head: *mut LIST_ENTRY,
    first: bool,
}

impl LdrModuleIterator {
    pub fn new(peb: ProcessEnvironmentBlock) -> Self {
        unsafe {
            let ldr = (*peb.Ldr).InMemoryOrderModuleList;
            let head = &(*peb.Ldr).InMemoryOrderModuleList as *const _ as *mut LIST_ENTRY;
            
            Self {
                current: ldr.Flink,
                head,
                first: true,
            }
        }
    }
}

impl Iterator for LdrModuleIterator {
    type Item = LdrModuleEntry;
    
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.current.is_null() {
                return None;
            }
            
            if !self.first && self.current == self.head {
                return None;
            }
            self.first = false;
            
            let entry_ptr = (self.current as usize - core::mem::offset_of!(LDR_DATA_TABLE_ENTRY, InMemoryOrderLinks)) as *mut LDR_DATA_TABLE_ENTRY;
            let entry = &*entry_ptr;
            
            let result = LdrModuleEntry::new(entry);
            
            self.current = (*self.current).Flink;
            Some(result)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ModuleBase(pub usize);

impl ModuleBase {
    pub fn as_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ModuleSize(pub usize);

#[derive(Debug)]
pub struct ModuleName(pub [u8; 256], pub usize);

impl fmt::Display for ModuleName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(s) = self.as_str() {
            write!(f, "{}", s)
        } else {
            write!(f, "")
        }
    }
}

impl PartialEq<str> for ModuleName {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == Some(other)
    }
}

impl PartialEq<&str> for ModuleName {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == Some(*other)
    }
}

impl PartialEq for ModuleName {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl ModuleName {
    pub fn new(u16_slice: &[u16]) -> Self {
        let mut buf = [0u8; 256];
        let mut len = 0;
        
        for chr in char::decode_utf16(u16_slice.iter().copied()) {
            if let Ok(c) = chr {
                let bytes = c.encode_utf8(&mut buf[len..]);
                len += bytes.len();
                if len >= 256 - 4 {
                    break;
                }
            }
        }
        
        Self(buf, len)
    }
    
    pub fn as_str(&self) -> Option<&str> {
        if self.1 == 0 {
            return None;
        }
        core::str::from_utf8(&self.0[..self.1]).ok()
    }
}

pub struct LdrEntry {
    pub base: ModuleBase,
    pub size: ModuleSize,
    pub name: ModuleName,
}

impl LdrEntry {
    pub fn base_address(&self) -> usize {
        self.base.0
    }
    
    pub fn size_of_image(&self) -> usize {
        self.size.0
    }
    
    pub fn name_str(&self) -> Option<&str> {
        self.name.as_str()
    }
}

pub struct LdrIterator {
    current: *mut LIST_ENTRY,
    head: *mut LIST_ENTRY,
    first: bool,
}

impl LdrIterator {
    pub fn new(peb: ProcessEnvironmentBlock) -> Self {
        unsafe {
            let ldr = (*peb.Ldr).InMemoryOrderModuleList;
            let head = &(*peb.Ldr).InMemoryOrderModuleList as *const _ as *mut LIST_ENTRY;
            
            Self {
                current: ldr.Flink,
                head,
                first: true,
            }
        }
    }
}

impl Iterator for LdrIterator {
    type Item = LdrEntry;
    
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.current.is_null() {
                return None;
            }
            
            if !self.first && self.current == self.head {
                return None;
            }
            self.first = false;
            
            let entry_ptr = (self.current as usize - core::mem::offset_of!(LDR_DATA_TABLE_ENTRY, InMemoryOrderLinks)) as *mut LDR_DATA_TABLE_ENTRY;
            let entry = &*entry_ptr;
            
            let name = if !entry.BaseDllName.Buffer.is_null() {
                let len = (entry.BaseDllName.Length / 2) as usize;
                let u16_slice = core::slice::from_raw_parts(entry.BaseDllName.Buffer, len);
                ModuleName::new(u16_slice)
            } else {
                ModuleName::new(&[])
            };
            
            let result = LdrEntry {
                base: ModuleBase(entry.DllBase as usize),
                size: ModuleSize(entry.SizeOfImage as usize),
                name,
            };
            
            self.current = (*self.current).Flink;
            Some(result)
        }
    }
}