use core::fmt::{self, Display, Formatter};
use core::ptr::null_mut;
use ntapi::ntrtl::*;
use winapi::shared::ntdef::{NTSTATUS, PWSTR, UNICODE_STRING};

pub fn write_utf16(f: &mut Formatter<'_>, slice: &[u16]) -> fmt::Result {
    let mut utf8_buf = [0u8; 4];
    let mut i = 0;
    
    while i < slice.len() {
        let mut buf = [0u16; 2];
        buf[0] = slice[i];
        let mut idx = 1;
        
        if i + 1 < slice.len() && (slice[i] & 0xFC00) == 0xD800 {
            buf[1] = slice[i + 1];
            idx = 2;
            i += 1;
        }
        
        if let Some(ch) = char::decode_utf16(buf[..idx].iter().cloned()).next() {
            if let Ok(c) = ch {
                let utf8_str = c.encode_utf8(&mut utf8_buf);
                f.write_str(utf8_str)?;
            }
        }
        i += 1;
    }
    
    Ok(())
}

pub fn find_variable<'a>(data: &'a [u16], name: &[u16]) -> Option<(usize, usize, &'a [u16], &'a [u16])> {
    let mut pos = 0;
    
    while pos < data.len() {
        let start = pos;
        
        while pos < data.len() && data[pos] != 0 {
            pos += 1;
        }
        
        if pos == start {
            break;
        }
        
        let var_slice = &data[start..pos];
        let mut sep = start;
        
        while sep < pos && data[sep] != (b'=' as u16) {
            sep += 1;
        }
        
        if sep < pos {
            let name_len = sep - start;
            if &var_slice[..name_len] == name {
                let value_start = sep + 1;
                return Some((start, pos + 1, &var_slice[..name_len], &var_slice[value_start - start..]));
            }
        }
        
        pos += 1;
    }
    
    None
}

pub fn count_variables(data: &[u16]) -> usize {
    let mut pos = 0;
    let mut count = 0;
    
    while pos < data.len() {
        let start = pos;
        while pos < data.len() && data[pos] != 0 {
            pos += 1;
        }
        if pos == start {
            break;
        }
        count += 1;
        pos += 1;
    }
    
    count
}

pub fn is_empty_block(data: &[u16]) -> bool {
    count_variables(data) == 0
}

pub struct EnvironmentIter<'a> {
    data: &'a [u16],
    pos: usize,
}

impl<'a> EnvironmentIter<'a> {
    pub fn new(data: &'a [u16]) -> Self {
        Self { data, pos: 0 }
    }
}

impl<'a> Iterator for EnvironmentIter<'a> {
    type Item = (EnvironmentString<'a>, EnvironmentString<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.data.len() {
            let start = self.pos;
            
            while self.pos < self.data.len() && self.data[self.pos] != 0 {
                self.pos += 1;
            }
            
            if self.pos == start {
                break;
            }
            
            let var_slice = &self.data[start..self.pos];
            let mut sep = start;
            
            while sep < self.pos && self.data[sep] != (b'=' as u16) {
                sep += 1;
            }
            
            if sep < self.pos {
                let name_slice = &var_slice[..sep - start];
                let value_slice = &var_slice[sep + 1 - start..];
                
                self.pos += 1;
                return Some((EnvironmentString(name_slice), EnvironmentString(value_slice)));
            }
            
            self.pos += 1;
        }
        
        None
    }
}

pub struct EnvironmentString<'a>(pub &'a [u16]);

impl<'a> Display for EnvironmentString<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write_utf16(f, self.0)
    }
}

impl<'a> EnvironmentString<'a> {
    pub fn as_utf16(&self) -> &'a [u16] {
        self.0
    }
}

pub struct UnicodeStringWrapper {
    inner: UNICODE_STRING,
    initialized: bool,
}

impl UnicodeStringWrapper {
    pub fn new() -> Self {
        Self {
            inner: unsafe { core::mem::zeroed() },
            initialized: false,
        }
    }

    pub fn as_slice(&self) -> &[u16] {
        unsafe { core::slice::from_raw_parts(self.inner.Buffer, self.inner.Length as usize) }
    }

    pub fn as_slice_leaked(&self) -> &'static [u16] {
        unsafe { core::slice::from_raw_parts(self.inner.Buffer, self.inner.Length as usize) }
    }
    
    pub fn from_asciiz(&mut self, str: *const i8) -> bool {
        let success = unsafe { RtlCreateUnicodeStringFromAsciiz(&mut self.inner, str as _) };
        if success != 0 {
            self.initialized = true;
        }
        success != 0
    }
    
    pub fn as_mut_ptr(&mut self) -> *mut UNICODE_STRING {
        &mut self.inner
    }
    
    pub fn as_ptr(&self) -> *const UNICODE_STRING {
        &self.inner
    }
}

impl Drop for UnicodeStringWrapper {
    fn drop(&mut self) {
        if self.initialized {
            unsafe { RtlFreeUnicodeString(&mut self.inner); }
        }
    }
}

pub struct EnvironmentPtr {
    ptr: *mut winapi::ctypes::c_void,
    owns: bool,
}

impl EnvironmentPtr {
    pub fn new() -> Result<Self, NTSTATUS> {
        let mut ptr = null_mut();
        let status = unsafe { RtlCreateEnvironmentEx(null_mut(), &mut ptr, 0) };
        if status == 0 {
            Ok(Self { ptr, owns: true })
        } else {
            Err(status)
        }
    }
    
    pub unsafe fn from_existing(ptr: *mut winapi::ctypes::c_void) -> Self {
        Self { ptr, owns: false }
    }
    
    pub fn as_ptr(&self) -> *mut winapi::ctypes::c_void {
        self.ptr
    }
    
    pub fn as_const_ptr(&self) -> *const u16 {
        self.ptr as *const u16
    }
}

impl Drop for EnvironmentPtr {
    fn drop(&mut self) {
        if self.owns && !self.ptr.is_null() {
            unsafe { RtlDestroyEnvironment(self.ptr); }
        }
    }
}