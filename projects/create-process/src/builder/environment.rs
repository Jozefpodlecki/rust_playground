use core::ptr;
use winapi::ctypes::c_void;

use crate::builder::types::AlignedBuffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvironmentBuilderError {
    BufferOverflow,
    KeyNotFound,
    InvalidUtf16,
    NullPointer,
    StringTooLong,
}

pub struct EnvironmentBuilder<const N: usize> {
    length: usize,
    buffer: AlignedBuffer<N>,
}

impl<const N: usize> EnvironmentBuilder<N> {
    const MAX_KEY_LEN: usize = 128;
    const MAX_VALUE_LEN: usize = 128;

    pub fn new() -> Self {
        Self {
            length: 0,
            buffer: AlignedBuffer::new(),
        }
    }

    fn encode_utf16<const M: usize>(s: &str) -> Result<([u16; M], usize), EnvironmentBuilderError> {
        let mut buf = [0u16; M];
        let mut len = 0;
        for ch in s.encode_utf16() {
            if len >= M {
                return Err(EnvironmentBuilderError::StringTooLong);
            }
            buf[len] = ch;
            len += 1;
        }
        Ok((buf, len))
    }

    fn write_entry(&mut self, key: &[u16], value: &[u16]) -> Result<(), EnvironmentBuilderError> {
        let entry_size = (key.len() + 1 + value.len() + 1) * 2;

        if self.length + entry_size > N {
            return Err(EnvironmentBuilderError::BufferOverflow);
        }

        unsafe {
            let ptr = self.buffer.as_mut_ptr().add(self.length) as *mut u16;
            let mut idx = 0;

            for &ch in key {
                ptr.add(idx).write(ch);
                idx += 1;
            }
            ptr.add(idx).write(b'=' as u16);
            idx += 1;
            for &ch in value {
                ptr.add(idx).write(ch);
                idx += 1;
            }
            ptr.add(idx).write(0);
        }

        self.length += entry_size;
        Ok(())
    }

    fn read_until_null(data: *const u16, pos: &mut usize) -> usize {
        let start = *pos;
        unsafe {
            while data.add(*pos).read() != 0 {
                *pos += 1;
            }
        }
        start
    }

    fn find_equals(data: *const u16, start: usize, end: usize) -> usize {
        let mut eq_pos = start;
        unsafe {
            while eq_pos < end && data.add(eq_pos).read() != b'=' as u16 {
                eq_pos += 1;
            }
        }
        eq_pos
    }

    fn find_entry(&self, key: &[u16]) -> Option<(usize, usize)> {
        let mut pos = 0;
        unsafe {
            let data = self.buffer.as_ptr() as *const u16;
            while pos < self.length / 2 {
                let start = Self::read_until_null(data, &mut pos);
                let eq_pos = Self::find_equals(data, start, pos);
                
                if eq_pos >= pos {
                    pos += 1;
                    continue;
                }

                let key_slice = core::slice::from_raw_parts(data.add(start), eq_pos - start);
                if key_slice == key {
                    return Some((start, pos - start + 1));
                }
                pos += 1;
            }
        }
        None
    }

    fn remove_entry(&mut self, start: usize, len: usize) {
        let remaining = (self.length / 2) - start - len;
        unsafe {
            let data = self.buffer.as_mut_ptr() as *mut u16;
            ptr::copy(data.add(start + len), data.add(start), remaining);
        }
        self.length -= len * 2;
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<(), EnvironmentBuilderError> {
        let (key_buf, key_len) = Self::encode_utf16::<{ 128 }>(key)?;
        let (value_buf, value_len) = Self::encode_utf16::<{ 128 }>(value)?;

        if let Some((start, len)) = self.find_entry(&key_buf[..key_len]) {
            self.remove_entry(start, len);
        }

        self.write_entry(&key_buf[..key_len], &value_buf[..value_len])
    }

    pub fn remove(&mut self, key: &str) -> Result<(), EnvironmentBuilderError> {
        let (key_buf, key_len) = Self::encode_utf16::<{ 128 }>(key)?;
        if let Some((start, len)) = self.find_entry(&key_buf[..key_len]) {
            self.remove_entry(start, len);
            Ok(())
        } else {
            Err(EnvironmentBuilderError::KeyNotFound)
        }
    }

    pub fn from_raw_parts(env_ptr: *mut c_void, len: usize) -> Result<Self, EnvironmentBuilderError> {
        if env_ptr.is_null() {
            return Err(EnvironmentBuilderError::NullPointer);
        }

        if len > N {
            return Err(EnvironmentBuilderError::BufferOverflow);
        }

        let mut builder = Self::new();
        unsafe {
            ptr::copy_nonoverlapping(
                env_ptr as *const u8,
                builder.buffer.as_mut_ptr(),
                len,
            );
            builder.length = len;
        }

        Ok(builder)
    }

    pub fn from_ptr(env_ptr: *mut c_void) -> Result<Self, EnvironmentBuilderError> {
        if env_ptr.is_null() {
            return Err(EnvironmentBuilderError::NullPointer);
        }

        let data = env_ptr as *const u16;
        let mut pos = 0;

        unsafe {
            while data.add(pos).read() != 0 || data.add(pos + 1).read() != 0 {
                pos += 1;
            }
            pos += 2;
        }

        let total_bytes = pos * 2;
        if total_bytes > N {
            return Err(EnvironmentBuilderError::BufferOverflow);
        }

        let mut builder = Self::new();
        unsafe {
            ptr::copy_nonoverlapping(
                env_ptr as *const u8,
                builder.buffer.as_mut_ptr(),
                total_bytes,
            );
            builder.length = total_bytes;
        }

        Ok(builder)
    }

    pub fn clear(&mut self) {
        self.length = 0;
        unsafe {
            ptr::write_bytes(self.buffer.as_mut_ptr(), 0, N);
        }
    }

    pub fn finalize(&mut self) -> Result<(), EnvironmentBuilderError> {
        if self.length + 2 > N {
            return Err(EnvironmentBuilderError::BufferOverflow);
        }

        unsafe {
            let ptr = self.buffer.as_mut_ptr().add(self.length) as *mut u16;
            ptr.write(0);
            ptr.add(1).write(0);
            self.length += 2;
        }

        Ok(())
    }

    pub fn build(&mut self) -> Result<*mut c_void, EnvironmentBuilderError> {
        self.finalize()?;
        Ok(self.buffer.as_mut_ptr() as *mut c_void)
    }

    pub fn as_mut_ptr(&mut self) -> *mut c_void {
        self.buffer.as_mut_ptr() as _
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.buffer.as_ptr()
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
}

impl<const N: usize> Default for EnvironmentBuilder<N> {
    fn default() -> Self {
        Self::new()
    }
}