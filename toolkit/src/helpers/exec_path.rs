use core::fmt;

pub struct Utf16Path {
    data: *const u16,
    length: usize,
}

pub struct StackUtf16Path<const N: usize> {
    data: [u16; N],
    length: usize,
}

impl<const N: usize> StackUtf16Path<N> {
    pub const fn new() -> Self {
        Self {
            data: [0; N],
            length: 0,
        }
    }

    pub fn as_slice(&self) -> &[u16] {
        &self.data[..self.length]
    }

    pub fn as_ptr(&self) -> *const u16 {
        self.data.as_ptr()
    }

    pub fn to_path(&self) -> Utf16Path {
        Utf16Path {
            data: self.data.as_ptr(),
            length: self.length,
        }
    }
}

impl Utf16Path {
    pub const fn new(data: *const u16, length: usize) -> Self {
        Self { data, length }
    }

    pub fn parent(&self) -> Self {
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

    pub fn join<const N: usize>(&self, other: &str) -> StackUtf16Path<N> {
        // Convert str to UTF-16
        let mut utf16 = [0u16; N];
        let mut len = 0;
        
        for c in other.encode_utf16() {
            if len < N {
                utf16[len] = c;
                len += 1;
            } else {
                break;
            }
        }
        
        let other_path = Utf16Path {
            data: utf16.as_ptr(),
            length: len,
        };
        
        self.join_path::<N>(&other_path)
    }

    // Join with another Utf16Path
    pub fn join_path<const N: usize>(&self, other: &Utf16Path) -> StackUtf16Path<N> {
        let mut result = StackUtf16Path::<N>::new();
        
        if self.length == 0 {
            let len = core::cmp::min(other.length, N);
            result.data[..len].copy_from_slice(&other.as_slice()[..len]);
            result.length = len;
            return result;
        }
        
        if other.length == 0 {
            let len = core::cmp::min(self.length, N);
            result.data[..len].copy_from_slice(&self.as_slice()[..len]);
            result.length = len;
            return result;
        }
        
        let self_ends_with_sep = if self.length > 0 {
            let slice = self.as_slice();
            let last = slice[self.length - 1];
            last == b'\\' as u16 || last == b'/' as u16
        } else {
            false
        };
        
        let other_starts_with_sep = if other.length > 0 {
            let slice = other.as_slice();
            let first = slice[0];
            first == b'\\' as u16 || first == b'/' as u16
        } else {
            false
        };
        
        let mut new_len = self.length + other.length;
        let mut needs_separator = false;
        
        if !self_ends_with_sep && !other_starts_with_sep && self.length > 0 && other.length > 0 {
            new_len += 1;
            needs_separator = true;
        } else if self_ends_with_sep && other_starts_with_sep {
            new_len -= 1;
        }
        
        if new_len > N {
            new_len = N;
        }
        
        let mut idx = 0;
        let self_len = core::cmp::min(self.length, new_len);
        result.data[..self_len].copy_from_slice(&self.as_slice()[..self_len]);
        idx = self_len;
        
        if needs_separator && idx < new_len {
            result.data[idx] = b'\\' as u16;
            idx += 1;
        }
        
        let other_start = if self_ends_with_sep && other_starts_with_sep {
            1
        } else {
            0
        };
        
        let remaining = new_len - idx;
        let other_slice = other.as_slice();
        let other_len = core::cmp::min(other_slice.len() - other_start, remaining);
        
        if other_len > 0 {
            result.data[idx..idx + other_len]
                .copy_from_slice(&other_slice[other_start..other_start + other_len]);
        }
        
        result.length = new_len;
        result
    }

    fn find_extension_separator(&self) -> Option<usize> {
        let slice = self.as_slice();
        for i in (0..slice.len()).rev() {
            if slice[i] == b'.' as u16 {
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

    pub fn to_string_lossy<const N: usize>(&self) -> heapless::String<N> {
        let mut s = heapless::String::new();
        for c in char::decode_utf16(self.as_slice().iter().cloned()) {
            if let Ok(c) = c {
                let _ = s.push(c);
            }
        }
        s
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

    pub fn is_absolute(&self) -> bool {
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

    fn find_last_separator(&self) -> Option<usize> {
        let slice = self.as_slice();
        for i in (0..slice.len()).rev() {
            let ch = slice[i];
            if ch == b'\\' as u16 || ch == b'/' as u16 {
                return Some(i);
            }
        }
        None
    }

    fn find_last_dot(&self) -> Option<usize> {
        let slice = self.as_slice();
        for i in (0..slice.len()).rev() {
            if slice[i] == b'.' as u16 {
                return Some(i);
            }
        }
        None
    }

    pub fn display<const N: usize>(&self) -> Utf16PathDisplay<'_, N> {
        Utf16PathDisplay { path: self, _marker: core::marker::PhantomData }
    }
}

pub struct Utf16PathDisplay<'a, const N: usize> {
    path: &'a Utf16Path,
    _marker: core::marker::PhantomData<[u8; N]>,
}

impl<'a, const N: usize> fmt::Display for Utf16PathDisplay<'a, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.path.to_string_lossy::<N>();
        write!(f, "{}", s)
    }
}

pub struct ExecutablePath(Utf16Path);

impl ExecutablePath {
    pub const fn new(path: Utf16Path) -> Self {
        Self(path)
    }

    pub fn display<const N: usize>(&self) -> Utf16PathDisplay<'_, N> {
        Utf16PathDisplay { path: &self.0, _marker: core::marker::PhantomData }
    }

    pub fn parent(&self) -> Utf16Path {
        if let Some(pos) = self.0.find_last_separator() {
            Utf16Path {
                data: self.0.data,
                length: pos,
            }
        } else {
            Utf16Path {
                data: self.0.data,
                length: 0,
            }
        }
    }

    pub fn directory_name(&self) -> Utf16Path {
        let parent = self.parent();
        if let Some(pos) = parent.find_last_separator() {
            Utf16Path {
                data: unsafe { parent.data.add(pos + 1) },
                length: parent.length - pos - 1,
            }
        } else {
            parent
        }
    }

    pub fn path(&self) -> Utf16Path {
        Utf16Path {
            data: self.0.data,
            length: self.0.length,
        }
    }

    pub fn file_name(&self) -> Utf16Path {
        if let Some(pos) = self.0.find_last_separator() {
            Utf16Path {
                data: unsafe { self.0.data.add(pos + 1) },
                length: self.0.length - pos - 1,
            }
        } else {
            Utf16Path {
                data: self.0.data,
                length: self.0.length,
            }
        }
    }

    pub fn file_stem(&self) -> Utf16Path {
        let name = self.file_name();
        if let Some(pos) = name.find_extension_separator() {
            Utf16Path {
                data: name.data,
                length: pos,
            }
        } else {
            name
        }
    }

    pub fn extension(&self) -> Utf16Path {
        let name = self.file_name();
        if let Some(pos) = name.find_extension_separator() {
            Utf16Path {
                data: unsafe { name.data.add(pos + 1) },
                length: name.length - pos - 1,
            }
        } else {
            Utf16Path {
                data: name.data,
                length: 0,
            }
        }
    }

    pub fn as_slice(&self) -> &[u16] {
        unsafe { core::slice::from_raw_parts(self.0.data, self.0.length) }
    }

    pub fn as_ptr(&self) -> *const u16 {
        self.0.data
    }
    
}

impl fmt::Display for Utf16Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.to_string_lossy::<260>();
        write!(f, "{}", s)
    }
}

impl fmt::Display for ExecutablePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.0.to_string_lossy::<260>();
        write!(f, "{}", s)
    }
}