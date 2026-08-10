use core::{fmt, slice};
use core::fmt::{Display, Formatter};
use toolkit::println;

use crate::types::*;
use crate::utils::*;

pub struct FixedEnvironment<const N: usize>(heapless::Vec<u16, N>);

impl<const N: usize> Environment for FixedEnvironment<N> {
    type Key = &'static str;
    type Value = &'static str;
    type ViewValue<'a> = EnvironmentValue<'a>;

    fn set(&mut self, name: &Self::Key, value: &Self::Value) -> Result<(), EnvironmentError> {
        let mut temp: heapless::Vec<u16, 256> = heapless::Vec::new();
        temp.extend(name.encode_utf16());
        temp.push('=' as u16);
        temp.extend(value.encode_utf16());
        temp.push(0);

        if self.0.len() + temp.len() + 1 > N {
            return Err(EnvironmentError::BufferFull);
        }
        
        self.0.extend(temp);
        
        Ok(())
    }

    fn get<'a>(&'a mut self, name: &Self::Key) -> Option<Self::ViewValue<'a>> {
        let target: heapless::Vec<u16, 256> = name.encode_utf16().collect();
        
        if let Some((_, _, _, value_slice)) = find_variable(&self.0, &target) {
            Some(EnvironmentValue::new(value_slice))
        } else {
            None
        }
    }

    fn remove(&mut self, name: &Self::Key) -> Result<(), EnvironmentError> {
        let target: heapless::Vec<u16, 256> = name.encode_utf16().collect();
        
        let (remove_start, remove_end) = {
            let mut pos = 0;
            let mut remove_start = 0;
            let mut remove_end = 0;
            let mut found = false;
            
            while pos < self.0.len() {
                let start = pos;
                
                while pos < self.0.len() && self.0[pos] != 0 {
                    pos += 1;
                }
                
                if pos == start {
                    break;
                }
                
                let var_slice = &self.0[start..pos];
                
                let mut sep = start;
                while sep < pos && self.0[sep] != (b'=' as u16) {
                    sep += 1;
                }
                
                if sep < pos {
                    let name_len = sep - start;
                    
                    if var_slice[..name_len] == target[..] {
                        remove_start = start;
                        remove_end = pos + 1;
                        found = true;
                        break;
                    }
                }
                
                pos += 1;
            }
            
            if !found {
                return Err(EnvironmentError::NotFound);
            }
            
            (remove_start, remove_end)
        };
        
        let mut temp: heapless::Vec<u16, N> = heapless::Vec::new();
        temp.extend_from_slice(&self.0[remove_end..])
            .map_err(|_| EnvironmentError::BufferFull)?;
        
        self.0.truncate(remove_start);
        self.0.extend_from_slice(&temp)
            .map_err(|_| EnvironmentError::BufferFull)?;
        
        Ok(())
    }

    fn as_raw_ptr(&self) -> *mut winapi::ctypes::c_void {
        self.0.as_ptr() as *mut winapi::ctypes::c_void
    }

    fn destroy(self) -> Result<(), EnvironmentError> {
        Ok(())
    }
    
    fn is_empty(&self) -> bool {
        is_empty_block(&self.0)
    }
    
    fn len(&self) -> usize {
        count_variables(&self.0)
    }
    
    fn clear(&mut self) -> Result<(), EnvironmentError> {
        self.0.clear();
        Ok(())
    }
    
    fn as_slice(&self) -> &[u16] {
        self.0.as_slice()
    }
    
    fn capacity(&self) -> usize {
        N
    }
}

impl<const N: usize> FixedEnvironment<N> {
    pub fn new() -> Self {
        Self(heapless::Vec::new())
    }

    pub fn actual_capacity(&self) -> usize {
        self.as_slice().len()
    }

    pub fn from_slice(slice: &[u16]) -> Result<Self, EnvironmentError> {
        let mut buffer = heapless::Vec::new();
        buffer.extend_from_slice(slice)
            .map_err(|_| EnvironmentError::BufferFull)?;
        Ok(Self(buffer))
    }

    pub fn from_raw_parts(data: *const u16, len: usize) -> Result<Self, EnvironmentError> {
        if len > N {
            return Err(EnvironmentError::BufferFull);
        }
        
        let mut buffer = heapless::Vec::new();
        let slice = unsafe { core::slice::from_raw_parts(data, len) };

        buffer.extend_from_slice(slice)
            .map_err(|_| EnvironmentError::BufferFull)?;
        
        Ok(Self(buffer))
    }

    pub fn iter(&self) -> EnvironmentIter<'_> {
        EnvironmentIter::new(self.as_slice())
    }

    pub fn shrink<const M: usize>(self) -> Result<FixedEnvironment<M>, EnvironmentError> {
        if self.0.len() > M {
            return Err(EnvironmentError::BufferFull);
        }
        
        let mut new_buffer = heapless::Vec::new();
        new_buffer.extend_from_slice(&self.0)
            .map_err(|_| EnvironmentError::BufferFull)?;
        
        Ok(FixedEnvironment(new_buffer))
    }
}

impl<const N: usize> Display for FixedEnvironment<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut pos = 0;
        let mut first = true;
        
        while pos < self.0.len() {
            let start = pos;
            
            while pos < self.0.len() && self.0[pos] != 0 {
                pos += 1;
            }
            
            if pos == start {
                break;
            }
            
            if !first {
                f.write_str("\n")?;
            }
            first = false;
            
            let var_slice = &self.0[start..pos];
            let mut sep = start;
            
            while sep < pos && self.0[sep] != (b'=' as u16) {
                sep += 1;
            }
            
            if sep < pos {
                let name_slice = &var_slice[..sep - start];
                let value_slice = &var_slice[sep + 1 - start..];
                
                write_utf16(f, name_slice)?;
                f.write_str("=")?;
                write_utf16(f, value_slice)?;
            }
            
            pos += 1;
        }
        
        Ok(())
    }
}