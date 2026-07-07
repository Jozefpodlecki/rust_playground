// data_buf.rs
use core::mem::{self, MaybeUninit};
use core::ptr;

pub unsafe trait Pod: Copy {
    fn default() -> Self;
}

macro_rules! impl_pod {
    ( $($t:ty),* ) => {
        $( unsafe impl Pod for $t { fn default() -> Self { 0 } } )*
    }
}

impl_pod! { u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize }

pub unsafe trait DataBuf: Default {
    type Inner: Pod;

    fn as_ref(&self) -> &[MaybeUninit<Self::Inner>];
    fn as_mut(&mut self) -> &mut [MaybeUninit<Self::Inner>];

    fn extend(&mut self, len: usize) -> Result<(), ()>;

    fn round_to_words(bytes: usize) -> usize {
        (bytes + mem::size_of::<Self::Inner>() - 1) / mem::size_of::<Self::Inner>()
    }
}

pub struct Buf<T: Pod, const N: usize> {
    data: [MaybeUninit<T>; N],
    used: usize,
}

impl<T: Pod, const N: usize> Default for Buf<T, N> {
    fn default() -> Self {
        Buf {
            data: unsafe { MaybeUninit::uninit().assume_init() },
            used: 0,
        }
    }
}

unsafe impl<T: Pod, const N: usize> DataBuf for Buf<T, N> {
    type Inner = T;
    
    fn as_ref(&self) -> &[MaybeUninit<Self::Inner>] {
        &self.data[..self.used]
    }
    
    fn as_mut(&mut self) -> &mut [MaybeUninit<Self::Inner>] {
        &mut self.data[..self.used]
    }
    
    fn extend(&mut self, len: usize) -> Result<(), ()> {
        if self.used + len <= N {
            self.used += len;
            Ok(())
        } else {
            Err(())
        }
    }
}