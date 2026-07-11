use core::{fmt, slice};

use winapi::shared::ntdef::UNICODE_STRING;

pub struct UnicodeString(UNICODE_STRING);

impl fmt::Display for UnicodeString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.Buffer.is_null() || self.0.Length == 0 {
            return write!(f, "");
        }
        
        let len = (self.0.Length / 2) as usize;
        let wide_slice = unsafe {
            slice::from_raw_parts(self.0.Buffer, len)
        };
        
        for c in char::decode_utf16(wide_slice.iter().cloned()) {
            match c {
                Ok(c) => {
                    write!(f, "{}", c);
                }
                Err(err) => {
                    continue;
                }
            }
        }

        Ok(())
    }
}