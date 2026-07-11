use core::fmt;

use winapi::ctypes::c_void;

pub struct Environment(pub *mut c_void);

impl Environment {
    pub fn path(&self) -> EnvironmentPathIter<'_> {
        let path_var = self.iter().find(|(key, _)| {
            key.to_string::<4>() == "Path"
        });
        
        EnvironmentPathIter {
            data: path_var.map(|(_, value)| value.as_u16_slice()).unwrap_or(&[]),
            pos: 0,
        }
    }

    pub fn iter(&self) -> EnvironmentIter<'_> {
        EnvironmentIter::new(self.0)
    }
}

pub struct EnvironmentPathIter<'a> {
    data: &'a [u16],
    pos: usize,
}

impl<'a> Iterator for EnvironmentPathIter<'a> {
    type Item = EnvironmentString<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.data.len() {
            return None;
        }

        let start = self.pos;
        let mut pos = self.pos;

        while pos < self.data.len() && self.data[pos] != b';' as u16 {
            pos += 1;
        }

        self.pos = pos + 1;

        if start == pos {
            return None;
        }

        Some(EnvironmentString(&self.data[start..pos]))
    }
}

pub struct EnvironmentIter<'a> {
    data: *const u16,
    pos: usize,
    _marker: core::marker::PhantomData<&'a ()>,
}

impl<'a> EnvironmentIter<'a> {
    pub fn new(env: *mut c_void) -> Self {
        Self {
            data: env as *const u16,
            pos: 0,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<'a> Iterator for EnvironmentIter<'a> {
    type Item = (EnvironmentString<'a>, EnvironmentString<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.is_null() {
            return None;
        }

        let start = self.pos;

        unsafe {
            let mut pos = start;
            while self.data.add(pos).read() != 0 {
                pos += 1;
            }

            if self.data.add(start).read() == 0 {
                self.pos = pos + 1;
                return None;
            }

            let mut eq_pos = start;
            while self.data.add(eq_pos).read() != b'=' as u16 && eq_pos < pos {
                eq_pos += 1;
            }

            if eq_pos >= pos {
                self.pos = pos + 1;
                return None;
            }

            let name = EnvironmentString(core::slice::from_raw_parts(self.data.add(start), eq_pos - start));
            let value = EnvironmentString(core::slice::from_raw_parts(self.data.add(eq_pos + 1), pos - eq_pos - 1));

            self.pos = pos + 1;
            Some((name, value))
        }
    }
}

pub struct EnvironmentString<'a>(&'a [u16]);

impl<'a> EnvironmentString<'a> {
    pub fn as_u16_slice(&self) -> &'a [u16] {
        self.0
    }
    
    pub fn to_string<const N: usize>(&self) -> heapless::String<N> {
        let mut s = heapless::String::new();
        for c in char::decode_utf16(self.0.iter().cloned()) {
            if let Ok(c) = c {
                let _ = s.push(c);
            }
        }
        s
    }
}

impl<'a> AsRef<[u16]> for EnvironmentString<'a> {
    fn as_ref(&self) -> &[u16] {
        self.0
    }
}

impl<'a> fmt::Display for EnvironmentString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in char::decode_utf16(self.0.iter().cloned()) {
            if let Ok(c) = c {
                write!(f, "{}", c)?;
            }
        }
        Ok(())
    }
}