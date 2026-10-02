use core::{mem, ptr::null_mut};

use winapi::{
    shared::ntdef::{OBJ_CASE_INSENSITIVE, OBJECT_ATTRIBUTES, UNICODE_STRING}, um::winnt::HANDLE,
};

const NULL_UNICODE_STRING: UNICODE_STRING = UNICODE_STRING {
    Length: 0,
    MaximumLength: 0,
    Buffer: null_mut(),
};

#[cfg(feature = "alloc")]
use crate::types::UnicodeString;

#[cfg(feature = "alloc")]
pub struct ObjectAttributes {
    name: Option<UnicodeString>,
    raw_name: UNICODE_STRING,
    root: HANDLE,
    attributes: u32,
}

#[cfg(feature = "alloc")]
impl ObjectAttributes {
    pub const fn new() -> Self {
        Self {
            name: None,
            raw_name: NULL_UNICODE_STRING,
            root: null_mut(),
            attributes: 0,
        }
    }

    pub fn name(mut self, name: UnicodeString) -> Self {
        self.raw_name = name.as_unicode_string();
        self.name = Some(name);
        self
    }

    pub const fn root(mut self, root: HANDLE) -> Self {
        self.root = root;
        self
    }

    pub const fn case_sensitive(mut self) -> Self {
        self.attributes &= !OBJ_CASE_INSENSITIVE;
        self
    }

    pub const fn case_insensitive(mut self) -> Self {
        self.attributes |= OBJ_CASE_INSENSITIVE;
        self
    }

    pub fn as_raw(&mut self) -> OBJECT_ATTRIBUTES {
        let object_name = if self.name.is_some() {
            &mut self.raw_name as *mut UNICODE_STRING
        } else {
            null_mut()
        };

        OBJECT_ATTRIBUTES {
            Length: mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: self.root,
            ObjectName: object_name,
            Attributes: self.attributes,
            SecurityDescriptor: null_mut(),
            SecurityQualityOfService: null_mut(),
        }
    }
}