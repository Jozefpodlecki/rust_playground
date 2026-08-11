use crate::encoder::types::*;
use heapless::Vec;

impl<const N: usize> BufferStorage for Vec<u8, N> {
    type Bytes = [u8; N];
    
    fn push(&mut self, byte: u8) -> EncoderResult<()> {
        self.push(byte).map_err(|_| EncoderError::BufferOverflow {
            capacity: N,
            needed: self.len() + 1,
        })
    }
    
    fn extend(&mut self, bytes: &[u8]) -> EncoderResult<()> {
        self.extend_from_slice(bytes).map_err(|_| EncoderError::BufferOverflow {
            capacity: N,
            needed: self.len() + bytes.len(),
        })
    }

    #[allow(unconditional_recursion)]
    fn len(&self) -> usize {
        Vec::<u8, N>::len(&self)
    }
    
    fn capacity(&self) -> usize {
        self.capacity()
    }
    
    fn clear(&mut self) {
        self.clear()
    }
    
    fn as_slice(&self) -> &[u8] {
        self.as_slice()
    }
    
    fn as_mut_slice(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }
}

impl<const N: usize> FixupStorage for Vec<Fixup, N> {
    type Fixups = [Fixup; N];
    
    fn push(&mut self, fixup: Fixup) -> EncoderResult<()> {
        self.push(fixup).map_err(|_| EncoderError::FixupOverflow {
            capacity: N,
            needed: self.len() + 1,
        })
    }

    #[allow(unconditional_recursion)]
    fn len(&self) -> usize {
       Vec::<Fixup, N>::len(&self)
    }
    
    fn iter(&self) -> core::slice::Iter<'_, Fixup> {
        self.as_slice().iter()
    }
    
    fn clear(&mut self) {
        self.clear()
    }
}
