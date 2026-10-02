use core::fmt;

#[cfg(feature = "alloc")]
use alloc::string::String;

use winapi::shared::ntdef::UNICODE_STRING;

use crate::types::Utf16Path;

pub struct ExecutablePath(Utf16Path);

impl ExecutablePath {
    pub const fn new(path: Utf16Path) -> Self {
        Self(path)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn parent(&self) -> Option<Utf16Path> {
        if let Some(pos) = self.0.find_last_separator() {
            Some(Utf16Path::new(
                self.0.data(),
                pos
            ))
        } else {
            None
        }
    }

    pub fn directory_name(&self) -> Option<Utf16Path> {
        let parent = self.parent()?;
        if let Some(pos) = parent.find_last_separator() {
            Some(Utf16Path::new(
                unsafe { parent.data().add(pos + 1) },
                parent.len() - pos - 1,
            ))
        } else {
            Some(parent)
        }
    }

    pub fn path(&self) -> Utf16Path {
        self.0
    }

    pub fn file_name(&self) -> Utf16Path {
        if let Some(pos) = self.0.find_last_separator() {
            Utf16Path::new(
                unsafe { self.0.data().add(pos + 1) },
                self.0.len() - pos - 1,
            )
        } else {
            Utf16Path::new(self.0.data(), self.0.len())
        }
    }

    pub fn file_stem(&self) -> Utf16Path {
        let name = self.file_name();
        if let Some(pos) = name.find_extension_separator() {
            Utf16Path::new(name.data(), pos)
        } else {
            name
        }
    }

    pub fn extension(&self) -> Utf16Path {
        let name = self.file_name();
        if let Some(pos) = name.find_extension_separator() {
            Utf16Path::new(
                unsafe { name.data().add(pos + 1) },
                name.len() - pos - 1,
            )
        } else {
            Utf16Path::new(name.data(), 0)
        }
    }

    pub fn as_slice(&self) -> &[u16] {
        unsafe { core::slice::from_raw_parts(self.0.data(), self.0.len()) }
    }

    pub fn as_ptr(&self) -> *const u16 {
        self.0.data()
    }
    
}

impl fmt::Display for ExecutablePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl fmt::Debug for ExecutablePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}