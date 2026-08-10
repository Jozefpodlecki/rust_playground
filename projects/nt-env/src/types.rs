use core::fmt::{self, Display};
use core::fmt::{Formatter};
use winapi::shared::ntdef::NTSTATUS;

#[derive(Debug, Clone, Copy)]
pub enum EnvironmentError {
    BufferFull,
    NotFound,
    InvalidName,
    InvalidValue,
    OutOfMemory,
    NotSupported,
    NtStatus(NTSTATUS),
}

pub trait Environment {
    type Key;
    type Value;
    type ViewValue<'a> where Self: 'a;

    fn set(&mut self, name: &Self::Key, value: &Self::Value) -> Result<(), EnvironmentError>;
    fn get<'a>(&'a mut self, name: &Self::Key) -> Option<Self::ViewValue<'a>>;
    fn remove(&mut self, name: &Self::Key) -> Result<(), EnvironmentError>;
    fn as_raw_ptr(&self) -> *mut winapi::ctypes::c_void;
    fn destroy(self) -> Result<(), EnvironmentError>;
    fn as_slice(&self) -> &[u16];
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
    fn capacity(&self) -> usize;
    fn clear(&mut self) -> Result<(), EnvironmentError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct EnvironmentValue<'a>(&'a [u16]);

impl<'a> EnvironmentValue<'a> {
    pub fn new(data: &'a [u16]) -> Self {
        Self(data)
    }

    pub fn as_ptr(&self) -> *const [u16] {
        self.0
    }
    
    pub fn as_utf16(&self) -> &'a [u16] {
        self.0
    }
}

impl<'a> Display for EnvironmentValue<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut pos = 0;
        let mut utf8_buf = [0u8; 4];
        
        while pos < self.0.len() {
            let mut buf = [0u16; 2];
            let mut idx = 0;
            buf[idx] = self.0[pos];
            idx += 1;
            
            if let Some(ch) = char::decode_utf16(buf[..idx].iter().cloned()).next() {
                if let Ok(c) = ch {
                    let utf8_str = c.encode_utf8(&mut utf8_buf);
                    f.write_str(utf8_str)?;
                }
            }
            pos += 1;
        }
        
        Ok(())
    }
}