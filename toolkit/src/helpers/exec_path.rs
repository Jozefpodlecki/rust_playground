use core::fmt;

pub struct Utf16Path {
    data: *const u16,
    length: usize,
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