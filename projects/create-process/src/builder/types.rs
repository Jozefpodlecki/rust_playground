
#[repr(C, align(8))]
pub struct AlignedBuffer<const N: usize>([u8; N]);

impl<const N: usize> AlignedBuffer<N> {
    pub fn new() -> Self {
        Self([0; N])
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.0.as_mut_ptr()
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.0.as_ptr()
    }

    pub fn len(&self) -> usize {
        N
    }
}