#[repr(C)]
pub struct CodeBuffer<const N: usize> {
    pub data: [u8; N],
    pub offset: usize,
    pub base: u64,
}

impl<const N: usize> CodeBuffer<N> {
    pub fn new(base: u64) -> Self {
        Self {
            data: [0; N],
            offset: 0,
            base,
        }
    }

    pub fn rva(&self) -> u32 {
        (self.as_ptr() as u64 - self.base) as u32
    }

    pub fn rva_at(&self, offset: usize) -> u32 {
        unsafe { (self.as_ptr().add(offset) as u64 - self.base) as u32 }
    }

    pub fn push_bytes(&mut self, bytes: &[u8]) -> &mut Self {
        for byte in bytes {
            self.push(*byte);
        }
        self
    }

    pub fn push(&mut self, byte: u8) -> &mut Self {
        self.data[self.offset] = byte;
        self.offset += 1;
        self
    }

    pub fn push_u64(&mut self, value: u64) -> &mut Self {
        self.push_bytes(&value.to_le_bytes())
    }

    pub fn push_u32(&mut self, value: u32) -> &mut Self {
        self.push_bytes(&value.to_le_bytes())
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }

    pub fn len(&self) -> usize {
        self.offset
    }

    pub fn remaining(&self) -> usize {
        N - self.offset
    }

    pub fn capacity(&self) -> usize {
        N
    }

    pub fn reset(&mut self) -> &mut Self {
        self.offset = 0;
        self
    }

    pub fn align(&mut self, alignment: usize) -> &mut Self {
        let padding = (alignment - (self.offset % alignment)) % alignment;
        for _ in 0..padding {
            self.push(0x90);
        }
        self
    }
}
