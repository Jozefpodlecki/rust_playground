use core::{fmt, slice};

use ntapi::{ntexapi::{SYSTEM_BASIC_INFORMATION, SystemBasicInformation}, ntmmapi::{MEMORY_INFORMATION_CLASS, MemoryBasicInformation, MemoryMappedFilenameInformation}};
use toolkit::{println, syscalls::NtQueryVirtualMemory};
use winapi::{shared::{basetsd::SIZE_T, ntdef::UNICODE_STRING}, um::winnt::*};

use crate::exports::ModuleExportsIter;

pub struct MemoryInformation {
    inner: MEMORY_BASIC_INFORMATION,
    mapped_name: MemoryMappedFilePath,
}

impl MemoryInformation {
    pub fn new(handle: HANDLE, info: MEMORY_BASIC_INFORMATION, mapped_name: MemoryMappedFilePath) -> Self {
        
        Self {
            inner: info,
            mapped_name,
        }
    }

    pub fn size_of_image(&self) -> usize {
        unsafe {
            let base = self.inner.BaseAddress as *const u8; 
            let dos_header = base as *const IMAGE_DOS_HEADER;
            let nt_headers_offset = (*dos_header).e_lfanew as usize;
            let nt_headers = base.add(nt_headers_offset) as *const IMAGE_NT_HEADERS64;
         
            (*nt_headers).OptionalHeader.SizeOfImage as usize
        }
    }

    pub fn exports(&self) -> Option<ModuleExportsIter> {
        ModuleExportsIter::new(self.inner.BaseAddress)
    }

    pub fn base_address(&self) -> *mut winapi::ctypes::c_void {
        self.inner.BaseAddress
    }
    
    pub fn file_path(&self) -> &str {
        self.mapped_name.as_str()
    }

    pub fn file_name(&self) -> &str {
        self.mapped_name.file_name()
    }

    pub fn extension(&self) -> &str {
        self.mapped_name.extension()
    }
}

pub struct NtModuleIterator {
    handle: HANDLE,
    address: PVOID,
    last_mapped_file_name: Option<MemoryMappedFilePath>
}

impl NtModuleIterator {
    pub fn new(handle: HANDLE) -> Self {
        Self {
            handle,
            address: core::ptr::null_mut(),
            last_mapped_file_name: None
        }
    }

    pub fn from_address(handle: HANDLE, start_address: PVOID) -> Self {
        Self {
            handle,
            address: start_address,
            last_mapped_file_name: None
        }
    }
}

impl Iterator for NtModuleIterator {
    type Item = MemoryInformation;
    
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            loop {
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

                let mapped_file_name = get_mapped_file_name(self.handle, mbi.BaseAddress);

                match mapped_file_name {
                    Some(file_name) => {
                        let ext = file_name.extension();
                        if ext != "exe" && ext != "dll" {
                            self.address = (base as usize).saturating_add(size) as PVOID;
                            continue;
                        }

                        if let Some(last) = &self.last_mapped_file_name {
                            if last == &file_name {
                                self.address = (base as usize).saturating_add(size) as PVOID;
                                continue;
                            }
                        }

                        self.last_mapped_file_name = Some(file_name);
                        self.address = (base as usize).saturating_add(size) as PVOID;
                        return Some(MemoryInformation::new(self.handle, mbi, file_name));
                    }
                    None => {
                        self.address = (base as usize).saturating_add(size) as PVOID;
                        self.last_mapped_file_name = None;
                        continue;
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct MemoryMappedFilePath {
    buffer: [u8; 256],
    len: usize
}

impl MemoryMappedFilePath {
    pub fn new() -> Self {
        Self {
            buffer: [0u8; 256],
            len: 0
        }
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.buffer.as_mut_ptr()
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.buffer.as_ptr()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn file_path(&self) -> &str {
        self.as_str()
    }

    pub fn file_name(&self) -> &str {
        let s = self.as_str();
        if let Some(pos) = s.rfind('\\') {
            &s[pos + 1..]
        } else {
            s
        }
    }

    pub fn extension(&self) -> &str {
        let name = self.file_name();
        let pos = name.rfind('.').unwrap();
        &name[pos + 1..]
    }

    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_raw_parts(self.buffer.as_ptr(), self.len) }
    }

    pub fn from_raw_parts(buffer: *mut u16, len: usize) -> Self {
        unsafe {
            let slice = core::slice::from_raw_parts(buffer as *const u16, len);
            let chars = char::decode_utf16(slice.iter().cloned());
            
            let mut result_buffer = [0u8; 256];
            let mut pos = 0;
            
            for c in chars {
                if let Ok(ch) = c {
                    let mut utf8_buf = [0u8; 4];
                    let encoded = ch.encode_utf8(&mut utf8_buf);
                    let bytes = encoded.as_bytes();
                    
                    if pos + bytes.len() <= result_buffer.len() {
                        result_buffer[pos..pos + bytes.len()].copy_from_slice(bytes);
                        pos += bytes.len();
                    } else {
                        break;
                    }
                }
            }
            
            Self {
                buffer: result_buffer,
                len: pos
            }
        }
    }
}

pub fn get_mapped_file_name(handle: HANDLE, base_address: PVOID) -> Option<MemoryMappedFilePath> {
    unsafe {
        let mut buffer = [0u8; 256];
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
        
        let us = &*(buffer.as_ptr() as *const UNICODE_STRING);
        let len = (us.Length as usize) / 2;
        
        Some(MemoryMappedFilePath::from_raw_parts(us.Buffer, len))
    }
}