pub trait Storage {
    type Buffer: AsRef<[u8]> + AsMut<[u8]>;
    fn new() -> Self::Buffer;
}

#[cfg(feature = "alloc")]
pub struct AllocStorage;

#[cfg(feature = "alloc")]
impl Storage for AllocStorage {
    type Buffer = alloc::alloc::Vec<u8>;
    fn new() -> Self::Buffer {
        alloc::vec::Vec::new()
    }
}

pub struct FixedStorage<const N: usize>;
impl<const N: usize> Storage for FixedStorage<N> {
    type Buffer = heapless::Vec<u8, N>;
    fn new() -> Self::Buffer {
        heapless::Vec::new()
    }
}

pub struct Decoder<S: Storage> {
    buffer: S::Buffer,
    _marker: core::marker::PhantomData<S>,
}

impl<S: Storage> Decoder<S> {
    pub fn new() -> Self {
        Self {
            buffer: S::new(),
            _marker: core::marker::PhantomData,
        }
    }
}