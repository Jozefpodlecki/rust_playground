use core::fmt::{self, Display, Formatter};

use heapless::Vec;


#[repr(transparent)]
pub struct ByteBlock<const N: usize>([u8; N]);

// impl<const N: usize> From<Vec<u8, N>> for ByteBlock<N> {
//     fn from(vec: Vec<u8, N>) -> Self {
//         let mut arr = [0u8; N];
//         let copy_len = vec.len().min(N);
//         arr[..copy_len].copy_from_slice(&vec[..copy_len]);
//         ByteBlock(arr)
//     }
// }

impl<const N: usize, const M: usize> From<Vec<u8, M>> for ByteBlock<N> {
    fn from(vec: Vec<u8, M>) -> Self {
        let mut arr = [0u8; N];
        let copy_len = vec.len().min(N);
        arr[..copy_len].copy_from_slice(&vec[..copy_len]);
        ByteBlock(arr)
    }
}

impl<const N: usize> From<&[u8]> for ByteBlock<N> {
    fn from(slice: &[u8]) -> Self {
        let mut arr = [0u8; N];
        let copy_len = slice.len().min(N);
        arr[..copy_len].copy_from_slice(&slice[..copy_len]);
        ByteBlock(arr)
    }
}

impl<const N: usize> ByteBlock<N> {
    pub fn new() -> Self {
        Self([0u8; N])
    }

    pub fn from_ptr(ptr: *const u8, len: usize) -> Self {
        let mut arr = [0u8; N];
        let copy_len = len.min(N);
        unsafe {
            core::ptr::copy_nonoverlapping(ptr, arr.as_mut_ptr(), copy_len);
        }
        ByteBlock(arr)
    }

    pub fn into_inner(self) -> [u8; N] {
        self.0
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    
    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        &mut self.0
    }
    
    pub fn len(&self) -> usize {
        N
    }
    
    pub fn is_empty(&self) -> bool {
        N == 0
    }
    
    pub fn get(&self, index: usize) -> Option<u8> {
        if index < N {
            Some(self.0[index])
        } else {
            None
        }
    }
}

impl<const N: usize> Default for ByteBlock<N> {
    fn default() -> Self {
        Self([0u8; N])
    }
}

impl<const N: usize> Display for ByteBlock<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "ByteBlock ({} bytes):", N)?;
        
        for (i, chunk) in self.0.chunks(16).enumerate() {
            // Hex part
            write!(f, "{:04X}: ", i * 16)?;
            
            for byte in chunk.iter() {
                write!(f, "{:02X} ", byte)?;
            }
            
            let padding = 16 - chunk.len();
            for _ in 0..padding {
                write!(f, "   ")?;
            }
            
            write!(f, " |")?;
            for byte in chunk.iter() {
                let c = *byte;
                if c >= 0x20 && c <= 0x7E {
                    write!(f, "{}", c as char)?;
                } else {
                    write!(f, ".")?;
                }
            }
            for _ in 0..padding {
                write!(f, " ")?;
            }
            writeln!(f, "|")?;
        }
        
        Ok(())
    }
}

impl<const N: usize> AsRef<[u8]> for ByteBlock<N> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<const N: usize> AsMut<[u8]> for ByteBlock<N> {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl<const N: usize> core::ops::Deref for ByteBlock<N> {
    type Target = [u8; N];
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> core::ops::DerefMut for ByteBlock<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
