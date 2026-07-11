use core::fmt;

pub struct HexDisplay<'a>(pub &'a [u8]);

impl<'a> fmt::Display for HexDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x")?;
        for &b in self.0 {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

impl<'a> fmt::Debug for HexDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "b\"")?;
        for &b in self.0 {
            if b.is_ascii_alphanumeric() || b == b'_' {
                write!(f, "{}", b as char)?;
            } else {
                write!(f, "\\x{:02x}", b)?;
            }
        }
        write!(f, "\"")
    }
}

impl<const N: usize> U8CStackString<N> {
    pub fn to_hex(&self) -> HexDisplay<'_> {
        HexDisplay(self.as_slice())
    }

    pub fn to_hex_full(&self) -> HexDisplay<'_> {
        HexDisplay(&self.buf[..self.len])
    }
}

#[repr(C)]
pub struct U8CStackString<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> fmt::Display for U8CStackString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slice = self.as_slice();
        match core::str::from_utf8(slice) {
            Ok(s) => write!(f, "{}", s),
            Err(_) => {
                write!(f, "0x")?;
                for &b in slice {
                    write!(f, "{:02x}", b)?;
                }
                Ok(())
            }
        }
    }
}

impl<const N: usize> fmt::Debug for U8CStackString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slice = self.as_slice();
        match core::str::from_utf8(slice) {
            Ok(s) => write!(f, "\"{}\"", s),
            Err(_) => {
                write!(f, "b\"")?;
                for &b in slice {
                    if b.is_ascii_alphanumeric() || b == b'_' {
                        write!(f, "{}", b as char)?;
                    } else {
                        write!(f, "\\x{:02x}", b)?;
                    }
                }
                write!(f, "\"")
            }
        }
    }
}

impl<const N: usize> Default for U8CStackString<N> {
    fn default() -> Self {
        let mut buf = [0u8; N];
        buf[0] = 0;
        Self { buf, len: 0 }
    }
}

impl<const N: usize> U8CStackString<N> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_utf16_lossy(utf16: &[u16]) -> Self {
        let mut result = Self::new();
    
        for ch in char::decode_utf16(utf16.iter().copied()) {
            if let Ok(c) = ch {
                // Check for null terminator
                if c == '\0' {
                    break;
                }
                
                // Convert char to UTF-8 bytes
                let mut buf = [0u8; 4];
                let bytes = c.encode_utf8(&mut buf);
                result.push_str(bytes);
            }
            // Invalid surrogates are ignored by decode_utf16
        }
        
        result
    }

    pub fn from_str(value: &str) -> Option<Self> {
        let bytes = value.as_bytes();
        if bytes.len() + 1 > N {
            return None;
        }

        let mut buf = [0u8; N];
        buf[..bytes.len()].copy_from_slice(bytes);
        buf[bytes.len()] = 0;

        Some(Self {
            buf,
            len: bytes.len(),
        })
    }

    pub fn from_ptr(ptr: *const u8) -> Option<Self> {
        if ptr.is_null() {
            return None;
        }

        let mut buf = [0u8; N];
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
            if len == N - 1 {
                let next = *ptr.add(i);
                if next != 0 {
                    return None;
                }
            }
        }

        buf[len] = 0;
        Some(Self { buf, len })
    }

    pub fn push_str(&mut self, value: &str) -> bool {
        for byte in value.bytes() {
            if self.len + 1 >= N {
                return false;
            }
            self.buf[self.len] = byte;
            self.len += 1;
        }

        if self.len + 1 > N {
            return false;
        }
        self.buf[self.len] = 0;
        true
    }

    pub fn push(&mut self, byte: u8) -> bool {
        if self.len + 1 >= N {
            return false;
        }
        self.buf[self.len] = byte;
        self.len += 1;
        self.buf[self.len] = 0;
        true
    }

    pub fn clear(&mut self) {
        self.len = 0;
        self.buf[0] = 0;
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.buf.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.buf.as_mut_ptr()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.buf[..self.len]
    }

    pub fn substring<const M: usize>(&self, start: usize) -> Option<U8CStackString<M>> {
        let slice = self.as_slice();
        if start >= slice.len() {
            return None;
        }
        
        let mut result = U8CStackString::<M>::new();
        for &c in &slice[start..] {
            result.push(c);
        }
        Some(result)
    }

      pub fn contains(&self, pattern: &str) -> bool {
        let slice = self.as_slice();
        let pattern_bytes = pattern.as_bytes();
        
        if pattern_bytes.is_empty() || slice.len() < pattern_bytes.len() {
            return false;
        }
        
        slice.windows(pattern_bytes.len())
            .any(|window| window == pattern_bytes)
    }

    pub fn contains_u8(&self, byte: u8) -> bool {
        self.as_slice().contains(&byte)
    }

    pub fn starts_with(&self, prefix: &str) -> bool {
        let slice = self.as_slice();
        let prefix_bytes = prefix.as_bytes();
        
        if prefix_bytes.len() > slice.len() {
            return false;
        }
        
        &slice[..prefix_bytes.len()] == prefix_bytes
    }

    pub fn starts_with_u8(&self, prefix: &[u8]) -> bool {
        let slice = self.as_slice();
        
        if prefix.len() > slice.len() {
            return false;
        }
        
        &slice[..prefix.len()] == prefix
    }

    pub fn ends_with(&self, suffix: &str) -> bool {
        let slice = self.as_slice();
        let suffix_bytes = suffix.as_bytes();
        
        if suffix_bytes.len() > slice.len() {
            return false;
        }
        
        &slice[slice.len() - suffix_bytes.len()..] == suffix_bytes
    }

    pub fn ends_with_u8(&self, suffix: &[u8]) -> bool {
        let slice = self.as_slice();
        
        if suffix.len() > slice.len() {
            return false;
        }
        
        &slice[slice.len() - suffix.len()..] == suffix
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(self.as_slice()).unwrap_or("")
    }

    pub fn to_lowercase(&self) -> Self {
        let slice = self.as_slice();
        let mut result = Self::new();
        
        for &byte in slice {
            result.push(byte.to_ascii_lowercase());
        }
        
        result
    }

    pub fn to_uppercase(&self) -> Self {
        let slice = self.as_slice();
        let mut result = Self::new();
        
        for &byte in slice {
            result.push(byte.to_ascii_uppercase());
        }
        
        result
    }

}