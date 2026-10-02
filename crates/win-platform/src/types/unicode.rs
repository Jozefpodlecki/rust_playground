use core::ops::{Deref, DerefMut};
use core::ptr::null_mut;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

use ntapi::ntrtl::{RtlDosPathNameToNtPathName_U_WithStatus, RtlFreeUnicodeString};
use winapi::shared::ntdef::UNICODE_STRING;

use crate::{NtError};

#[cfg(feature = "alloc")]
pub struct UnicodeString(Box<[u16]>);

#[cfg(feature = "alloc")]
impl From<&str> for UnicodeString {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

#[cfg(feature = "alloc")]
impl From<alloc::string::String> for UnicodeString {
    fn from(s: alloc::string::String) -> Self {
        Self::from_str(&s)
    }
}

#[cfg(feature = "alloc")]
impl UnicodeString {
    #[cfg(feature = "alloc")]
    pub const fn new(buffer: Box<[u16]>) -> Self {
        Self(buffer)
    }

    #[cfg(feature = "alloc")]
    pub fn from_str(path: &str) -> Self {
        let buffer = path.encode_utf16().chain(core::iter::once(0)).collect::<Box<_>>();
        Self(buffer)
    }

    #[cfg(feature = "alloc")]
    pub fn from_raw_parts(ptr: *const u16, len: usize) -> Option<Self> {
        if ptr.is_null() {
            return None;
        }

        let buffer = unsafe { core::slice::from_raw_parts(ptr, len) }
            .iter()
            .copied()
            .collect::<Box<_>>();

        Some(Self(buffer))
    }

    /// Raw UTF-16 contents, no trailing NUL, no prefix.
    pub fn as_slice(&self) -> &[u16] {
        &self.0
    }

    /// Length in bytes (2 × code units), for UNICODE_STRING / NameLength fields.
    pub fn byte_len(&self) -> usize {
        self.0.len() * 2
    }

    /// Consume into a Box<[u16]>. Useful when you need to build
    /// another structure inline and can't borrow self.
    pub fn into_inner(self) -> Box<[u16]> {
        self.0
    }

    pub fn as_unicode_string(&self) -> UNICODE_STRING {
        let len = (self.0.len() - 1) * 2;
        UNICODE_STRING {
            Length: len as u16,
            MaximumLength: (self.0.len() * 2) as u16,
            Buffer: self.0.as_ptr() as _,
        }
    }

    pub fn canonicalise(&self) -> Result<Self, NtError> {
        use core::mem::zeroed;

        let mut nt_path: UNICODE_STRING = unsafe { zeroed() };
        let src = self.as_unicode_string();

        let status = unsafe {
            RtlDosPathNameToNtPathName_U_WithStatus(
                src.Buffer,
                &mut nt_path,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            )
        };

        if status != 0 {
            return Err(NtError(status));
        }

        let len = (nt_path.Length / 2) as usize;
        let slice = unsafe { core::slice::from_raw_parts(nt_path.Buffer, len) };
        let uc = slice.iter().copied().chain(core::iter::once(0)).collect::<Box<_>>();

        unsafe { RtlFreeUnicodeString(&mut nt_path) };

        Ok(Self(uc))
    }
}

#[cfg(feature = "alloc")]
impl core::fmt::Display for UnicodeString {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use core::fmt::Write;

        let slice: &[u16] = self.0.as_ref();
        for c in char::decode_utf16(slice.iter().copied()) {
            match c {
                Ok(c) => f.write_char(c)?,
                Err(_) => f.write_char(char::REPLACEMENT_CHARACTER)?,
            }
        }
        Ok(())
    }
}