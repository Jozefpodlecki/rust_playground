use core::fmt;

use winapi::shared::ntdef::UNICODE_STRING;

use crate::{U8CStackString, println, types::ToUnicode};

const MAX_PAT_LEN: usize = 128;

pub struct U16CStackString<const N: usize> {
    buf: [u16; N],
    len: usize,
}

impl<const N: usize> ToUnicode for U16CStackString<N> {
    fn as_unicode(&self) -> UNICODE_STRING {
        self.to_unicode_string()
    }
}

impl<const N: usize> Clone for U16CStackString<N> {
    fn clone(&self) -> Self {
        let mut buf = [0u16; N];
        buf[..self.len].copy_from_slice(&self.buf[..self.len]);
        Self { buf, len: self.len }
    }
}

impl<const N: usize> fmt::Display for U16CStackString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return write!(f, "");
        }
        let slice = self.as_slice();
        let string = U8CStackString::<N>::from_utf16_lossy(slice);
        write!(f, "{}", string)
    }
}

impl<const N: usize> fmt::Debug for U16CStackString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return write!(f, "\"\"");
        }
        let slice = self.as_slice();
        let string = U8CStackString::<N>::from_utf16_lossy(slice);
        write!(f, "{:?}", string)
    }
}

impl<const N: usize> Default for U16CStackString<N> {
    fn default() -> Self {
        let mut buf = [0u16; N];
        buf[0] = 0;
        Self { buf, len: 0 }
    }
}

impl<const N: usize> core::fmt::Write for U16CStackString<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if !self.push_str(s) {
            return Err(core::fmt::Error);
        }
        Ok(())
    }
}

impl<const N: usize> U16CStackString<N> {
    pub fn new() -> Self {
        Self::default()
    }

    // pub fn from_utf16_lossy(utf16: &[u16]) -> Self {
    //     let mut buf = [0u8; N];
    //     let mut len = 0;
        
    //     for &code_unit in utf16 {
    //         if code_unit == 0 {
    //             break;
    //         }
            
    //         // Simple lossy conversion - just handle ASCII and basic UTF-16
    //         if code_unit <= 0x7F {
    //             if len < N - 1 {
    //                 buf[len] = code_unit as u8;
    //                 len += 1;
    //             }
    //         } else if code_unit <= 0x7FF {
    //             if len + 2 < N - 1 {
    //                 buf[len] = (0xC0 | ((code_unit >> 6) & 0x1F)) as u8;
    //                 buf[len + 1] = (0x80 | (code_unit & 0x3F)) as u8;
    //                 len += 2;
    //             }
    //         } else if code_unit <= 0xFFFF {
    //             if len + 3 < N - 1 {
    //                 buf[len] = (0xE0 | ((code_unit >> 12) & 0x0F)) as u8;
    //                 buf[len + 1] = (0x80 | ((code_unit >> 6) & 0x3F)) as u8;
    //                 buf[len + 2] = (0x80 | (code_unit & 0x3F)) as u8;
    //                 len += 3;
    //             }
    //         } else {
    //             // Surrogate pairs or other - use replacement char
    //             if len < N - 1 {
    //                 buf[len] = 0xEF; // UTF-8 replacement
    //                 buf[len + 1] = 0xBF;
    //                 buf[len + 2] = 0xBD;
    //                 len += 3;
    //             }
    //         }
    //     }
        
    //     if len < N {
    //         buf[len] = 0;
    //     } else {
    //         buf[N - 1] = 0;
    //     }
        
    //     Self { buf, len }
    // }

    pub fn from_utf8_bytes(bytes: &[u8]) -> Option<Self> {
        let effective_len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        let trimmed = &bytes[..effective_len];

        let s = core::str::from_utf8(trimmed).ok()?;

        Self::from_str(s)
    }

    pub fn from_str(value: &str) -> Option<Self> {
        let mut buf = [0u16; N];
        let mut len = 0;
        
        for ch in value.encode_utf16() {
            if len + 1 >= N {
                return None;
            }
            buf[len] = ch;
            len += 1;
        }
        
        if len + 1 > N {
            return None;
        }
        buf[len] = 0;
        
        Some(Self { buf, len })
    }

    pub fn from_ptr(ptr: *const u16) -> Option<Self> {
        if ptr.is_null() {
            return None;
        }
        
        let mut buf = [0u16; N];
        let mut len = 0;
        
        unsafe {
            let mut i = 0;
            while len < N - 1 {
                let ch = *ptr.add(i);
                if ch == 0 {
                    break;
                }
                buf[len] = ch;
                len += 1;
                i += 1;
            }
        }
        
        buf[len] = 0;
        
        Some(Self { buf, len })
    }

    pub unsafe fn from_raw_parts(ptr: *const u16, len: usize) -> Option<Self> {
        if ptr.is_null() {
            return None;
        }
        
        if len >= N {
            return None;
        }
        
        let mut buf = [0u16; N];
        let slice = unsafe { core::slice::from_raw_parts(ptr, len) };
        buf[..len].copy_from_slice(slice);
        buf[len] = 0; // Null terminate
        
        Some(Self { buf, len })
    }


    pub fn as_u8_stack_string<const M: usize>(&self) -> U8CStackString<M> {
        let slice = self.as_slice();
        U8CStackString::<M>::from_utf16_lossy(slice)
    }

    pub fn push_str(&mut self, value: &str) -> bool {
        for ch in value.encode_utf16() {
            if self.len + 1 >= N {
                return false;
            }
            self.buf[self.len] = ch;
            self.len += 1;
        }
        
        if self.len + 1 > N {
            return false;
        }
        self.buf[self.len] = 0;
        
        true
    }

    pub fn push(&mut self, ch: u16) -> bool {
        if self.len + 1 >= N {
            return false;
        }
        self.buf[self.len] = ch;
        self.len += 1;
        self.buf[self.len] = 0;
        true
    }

    pub fn prepend(&mut self, prefix: &str) -> bool {
        let prefix_len = prefix.encode_utf16().count();
        
        if self.len + prefix_len + 1 > N {
            return false;
        }
        
        for i in (0..self.len).rev() {
            self.buf[i + prefix_len] = self.buf[i];
        }
        
        let mut idx = 0;
        for ch in prefix.encode_utf16() {
            self.buf[idx] = ch;
            idx += 1;
        }
        
        self.len += prefix_len;
        self.buf[self.len] = 0;
        
        true
    }
    
    pub fn clear(&mut self) {
        self.len = 0;
        self.buf[0] = 0;
    }
    
    pub fn as_ptr(&self) -> *const u16 {
        self.buf.as_ptr()
    }
    
    pub fn as_mut_ptr(&mut self) -> *mut u16 {
        self.buf.as_mut_ptr()
    }
    
    pub fn len(&self) -> usize {
        self.len
    }
    
    pub fn as_slice(&self) -> &[u16] {
        &self.buf[..self.len]
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn leak(&mut self) -> *mut u16 {
        self.buf.as_mut_ptr()
    }

    pub fn contains(&self, pat: &str) -> bool {
        if self.is_empty() || pat.is_empty() {
            return false;
        }

        let mut pat_utf16 = [0u16; MAX_PAT_LEN];
        let mut pat_len = 0;
        
        for ch in pat.encode_utf16() {
            if pat_len >= MAX_PAT_LEN {
                return false;
            }
            pat_utf16[pat_len] = ch;
            pat_len += 1;
        }
        
        if pat_len == 0 {
            return false;
        }

        let slice = self.as_slice();
        if slice.len() < pat_len {
            return false;
        }

        for i in 0..=slice.len() - pat_len {
            let mut found = true;
            for j in 0..pat_len {
                if slice[i + j] != pat_utf16[j] {
                    found = false;
                    break;
                }
            }
            if found {
                return true;
            }
        }
        false
    }

    pub fn contains_str(&self, pat: &str) -> bool {
        self.contains(pat)
    }

    pub fn contains_u16_slice(&self, pat: &[u16]) -> bool {
        if self.is_empty() || pat.is_empty() {
            return false;
        }

        let slice = self.as_slice();
        if slice.len() < pat.len() {
            return false;
        }

        for i in 0..=slice.len() - pat.len() {
            let mut found = true;
            for j in 0..pat.len() {
                if slice[i + j] != pat[j] {
                    found = false;
                    break;
                }
            }
            if found {
                return true;
            }
        }
        false
    }

    pub fn contains_u16(&self, ch: u16) -> bool {
        self.as_slice().contains(&ch)
    }

    pub fn to_unicode_string(&self) -> UNICODE_STRING {
        let len = (self.len * 2) as u16;
        UNICODE_STRING {
            Length: len,
            MaximumLength: len + 2,
            Buffer: self.as_ptr() as _,
        }
    }

    pub fn eq_ignore_ascii_case_str(&self, other: &str) -> bool {
        if self.is_empty() && other.is_empty() {
            return true;
        }
        
        let self_slice = self.as_slice();
        let other_bytes = other.as_bytes();
        
        let mut i = 0;
        let mut j = 0;
        
        while i < self.len && j < other_bytes.len() {
            let a = self_slice[i];
            let b = other_bytes[j] as u16;
            
            if a == b {
                i += 1;
                j += 1;
                continue;
            }
            
            if a >= 65 && a <= 90 {
                if a + 32 != b {
                    return false;
                }
            } else if a >= 97 && a <= 122 {
                if a - 32 != b {
                    return false;
                }
            } else {
                return false;
            }
            
            i += 1;
            j += 1;
        }
        
        i == self.len && j == other_bytes.len()
    }
}
