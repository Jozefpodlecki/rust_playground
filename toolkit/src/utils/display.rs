use core::fmt;

use winapi::shared::ntdef::{PLARGE_INTEGER, POBJECT_ATTRIBUTES, PUNICODE_STRING};

pub struct Utf16String<'a>(&'a [u16]);

impl<'a> Utf16String<'a> {
    pub fn new(buffer: *const u16, length: u16) -> Self {
        if buffer.is_null() || length == 0 {
            Self(&[])
        } else {
            let len = (length / 2) as usize;
            let slice = unsafe { core::slice::from_raw_parts(buffer, len) };
            Self(slice)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> PartialEq<&str> for Utf16String<'a> {
    fn eq(&self, other: &&str) -> bool {
        let mut buf = [0u16; 64];
        let mut len = 0;
        
        for c in other.encode_utf16() {
            
            if len >= buf.len() {
                return false;
            }

            buf[len] = c;
            len += 1;
        }
        
        self.0.len() == len && self.0.iter().zip(&buf[..len]).all(|(a, b)| a == b)
    }
}

impl<'a> fmt::Display for Utf16String<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            write!(f, "(empty)")?;
            return Ok(());
        }

        let iter = char::decode_utf16(self.0.iter().cloned())
            .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER));
            
        for char in iter {
            write!(f, "{}", char)?;
        }
        
        Ok(())
    }
}

pub struct UnicodeStringDisplay(PUNICODE_STRING);

impl UnicodeStringDisplay {
    pub fn new(value: PUNICODE_STRING) -> Self {
        Self(value)
    }
}


impl fmt::Display for UnicodeStringDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            if self.0.is_null() {
                write!(f, "(null)")?;
                return Ok(());
            }

            let us = &*self.0;
            writeln!(f, "========== UNICODE_STRING ==========")?;
            writeln!(f, "Length:          {}", us.Length)?;
            writeln!(f, "MaximumLength:   {}", us.MaximumLength)?;
            writeln!(f, "Buffer:          {:p}", us.Buffer)?;

            if !us.Buffer.is_null() && us.Length > 0 {
                writeln!(f, "{}", Utf16String::new(us.Buffer, us.Length))?;
            } else {
                writeln!(f, "(empty)")?;
            }
            writeln!(f, "===============================================")?;
        }
        Ok(())
    }
}

pub struct LargeIntegerDisplay(PLARGE_INTEGER);

impl LargeIntegerDisplay {
    pub fn new(li: PLARGE_INTEGER) -> Self {
        Self(li)
    }
}

impl fmt::Display for LargeIntegerDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            if self.0.is_null() {
                write!(f, "(null)")?;
            } else {
                let li = &*self.0;
                let value = *li.QuadPart();
                write!(f, "0x{:0X} ({})", value as u64, value)?;
            }
        }
        Ok(())
    }
}

pub struct ObjectAttributesDisplay(POBJECT_ATTRIBUTES);

impl ObjectAttributesDisplay {
    pub fn new(obj_attr: POBJECT_ATTRIBUTES) -> Self {
        Self(obj_attr)
    }
}

impl fmt::Display for ObjectAttributesDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            let obj_attr = &*self.0;
            writeln!(f, "========== ObjectAttributes ==========")?;
            writeln!(f, "Length:              {}", obj_attr.Length)?;
            writeln!(f, "RootDirectory:       {:p}", obj_attr.RootDirectory)?;
            writeln!(f, "ObjectName:          {:p}", obj_attr.ObjectName)?;
            writeln!(f, "Attributes:          0x{:X}", obj_attr.Attributes)?;
            writeln!(f, "SecurityDescriptor:  {:p}", obj_attr.SecurityDescriptor)?;
            writeln!(f, "SecurityQualityOfService: {:p}", obj_attr.SecurityQualityOfService)?;
            writeln!(f, "===============================================")?;

            if !obj_attr.ObjectName.is_null() {
                let name = &*obj_attr.ObjectName;
                writeln!(f, "========== UNICODE_STRING ==========")?;
                writeln!(f, "Length:     {}", name.Length)?;
                writeln!(f, "MaximumLength: {}", name.MaximumLength)?;
                writeln!(f, "Buffer:     {:p}", name.Buffer)?;
                
                if !name.Buffer.is_null() && name.Length > 0 {
                    writeln!(f, "{}", Utf16String::new(name.Buffer, name.Length))?;
                }
                writeln!(f, "===============================================")?;
            }
        }
        Ok(())
    }
}