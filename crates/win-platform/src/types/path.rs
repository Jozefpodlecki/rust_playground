use core::fmt::{self, Write};

#[cfg(feature = "alloc")]
use alloc::string::String;

use winapi::shared::ntdef::UNICODE_STRING;

#[derive(Clone, Copy)]
pub struct Utf16Path {
    data: *const u16,
    length: usize,
}

impl Utf16Path {
    pub const fn new(data: *const u16, length: usize) -> Self {
        Self { data, length }
    }

    pub const fn data(&self) -> *const u16 {
        self.data
    }

    pub const fn len(&self) -> usize {
        self.length
    }

    pub const fn parent(&self) -> Self {
        if let Some(pos) = self.find_last_separator() {
            Self {
                data: self.data,
                length: pos,
            }
        } else {
            Self {
                data: self.data,
                length: 0,
            }
        }
    }

    pub const fn starts_with_separator(&self) -> bool {
        let slice = self.as_slice();
        if slice.is_empty() {
            return false;
        }
        let first = slice[0];
        first == b'\\' as u16 || first == b'/' as u16
    }

    pub const fn ends_with_separator(&self) -> bool {
        let slice = self.as_slice();
        if slice.is_empty() {
            return false;
        }
        let last = slice[slice.len() - 1];
        last == b'\\' as u16 || last == b'/' as u16
    }

    pub const fn find_extension_separator(&self) -> Option<usize> {
        let slice = self.as_slice();
        let mut i = slice.len();
        while i > 0 {
            i -= 1;
            let ch = unsafe { *slice.get_unchecked(i) };
            if ch == b'.' as u16 {
                return Some(i);
            }
        }
        None
    }

    pub const fn as_slice(&self) -> &[u16] {
        if self.data.is_null() || self.length == 0 {
            &[]
        } else {
            unsafe { core::slice::from_raw_parts(self.data, self.length) }
        }
    }

    #[cfg(feature = "alloc")]
    pub fn to_string_lossy(&self) -> String {
        String::from_utf16_lossy(self.as_slice())
    }
    
    pub fn file_name(&self) -> Self {
        if let Some(pos) = self.find_last_separator() {
            Self {
                data: unsafe { self.data.add(pos + 1) },
                length: self.length - pos - 1,
            }
        } else {
            Self {
                data: self.data,
                length: self.length,
            }
        }
    }

    pub fn extension(&self) -> Self {
        let name = self.file_name();
        if let Some(pos) = name.find_last_dot() {
            Self {
                data: unsafe { name.data.add(pos + 1) },
                length: name.length - pos - 1,
            }
        } else {
            Self {
                data: self.data,
                length: 0,
            }
        }
    }

    pub fn file_stem(&self) -> Self {
        let name = self.file_name();
        if let Some(pos) = name.find_last_dot() {
            Self {
                data: name.data,
                length: pos,
            }
        } else {
            name
        }
    }

    pub const fn is_absolute(&self) -> bool {
        if self.length < 3 {
            return false;
        }
        let slice = self.as_slice();
        // Check for drive letter like C:\ or C:/
        (slice[0] >= b'A' as u16 && slice[0] <= b'Z' as u16 || 
         slice[0] >= b'a' as u16 && slice[0] <= b'z' as u16) &&
        slice[1] == b':' as u16 &&
        (slice[2] == b'\\' as u16 || slice[2] == b'/' as u16)
    }

    pub const fn find_last_separator(&self) -> Option<usize> {
        let slice = self.as_slice();
        let mut i = slice.len();
        while i > 0 {
            i -= 1;
            let ch = slice[i];
            if ch == b'\\' as u16 || ch == b'/' as u16 {
                return Some(i);
            }
        }
        None
    }

    pub fn find_last_dot(&self) -> Option<usize> {
        let slice = self.as_slice();
        for i in (0..slice.len()).rev() {
            if slice[i] == b'.' as u16 {
                return Some(i);
            }
        }
        None
    }

}

impl fmt::Display for Utf16Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in char::decode_utf16(self.as_slice().iter().copied()) {
            match c {
                Ok(c) => f.write_char(c)?,
                Err(_) => f.write_char(char::REPLACEMENT_CHARACTER)?,
            }
        }
        Ok(())
    }
}

impl fmt::Debug for Utf16Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('"')?;
        fmt::Display::fmt(self, f)?;
        f.write_char('"')
    }
}