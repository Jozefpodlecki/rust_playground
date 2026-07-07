use core::fmt;
use core::marker::PhantomData;
use core::mem::{self, MaybeUninit};
use core::ops::{Deref, DerefMut};
use core::ptr;

use crate::stack_trait::{DataBuf, Pod};

#[repr(C)]
pub struct Stacked<T: ?Sized, D: DataBuf> {
    data: D,
    _marker: PhantomData<T>,
}

impl<T: ?Sized, D: DataBuf> Stacked<T, D> {
    pub fn new<U>(value: U) -> Result<Self, U>
    where
        U: Sized + core::marker::Unsize<T>,
        D: Default,
    {
        let mut data = D::default();
        let size = mem::size_of::<U>();
        let meta = MetaInfo::from_ptr(&value as *const U as *const T);
        let req_words = D::round_to_words(meta.meta_len * mem::size_of::<usize>()) + D::round_to_words(size);
        
        if data.extend(req_words).is_err() {
            return Err(value);
        }
        
        unsafe {
            let buf = data.as_mut();
            let ptr = buf.as_mut_ptr() as *mut U;
            ptr.write(value);
            
            let info_ofs = buf.len() - D::round_to_words(meta.meta_len * mem::size_of::<usize>());
            let info_dst = &mut buf[info_ofs..];
            store_metadata(info_dst, &meta.meta[..meta.meta_len]);
        }
        
        Ok(Self {
            data,
            _marker: PhantomData,
        })
    }
    
    unsafe fn as_ptr(&self) -> *mut T {
        let buf = self.data.as_ref();
        let info_size = mem::size_of::<*mut T>() / mem::size_of::<usize>() - 1;
        let info_ofs = buf.len() - D::round_to_words(info_size * mem::size_of::<usize>());
        let (data, meta) = buf.split_at(info_ofs);
        make_fat_ptr(data.as_ptr() as *mut (), &meta)
    }
    
    unsafe fn as_ptr_mut(&mut self) -> *mut T {
        let buf = self.data.as_mut();
        let info_size = mem::size_of::<*mut T>() / mem::size_of::<usize>() - 1;
        let info_ofs = buf.len() - D::round_to_words(info_size * mem::size_of::<usize>());
        let (data, meta) = buf.split_at_mut(info_ofs);
        make_fat_ptr(data.as_mut_ptr() as *mut (), &meta)
    }
    
    pub fn into_inner(mut self) -> T
    where
        T: Sized,
    {
        unsafe { ptr::read(self.data.as_mut().as_mut_ptr() as *const T) }
    }
    
    pub fn capacity(&self) -> usize {
        self.data.as_ref().len() * mem::size_of::<D::Inner>()
    }
}

struct MetaInfo {
    data_ptr: *const (),
    meta_len: usize,
    meta: [usize; 3],
}

fn mem_as_slice<T>(ptr: &mut T) -> &mut [usize] {
    let words = mem::size_of::<T>() / mem::size_of::<usize>();
    unsafe { core::slice::from_raw_parts_mut(ptr as *mut _ as *mut usize, words) }
}


impl MetaInfo {
    pub fn from_ptr<T: ?Sized>(mut ptr: *const T) -> Self {
        let addr = ptr as *const ();
        let rv = mem_as_slice(&mut ptr);
        let mut vals = [0; 3];
        vals[..rv.len() - 1].copy_from_slice(&rv[1..]);
        
        MetaInfo {
            data_ptr: addr,
            meta_len: rv.len() - 1,
            meta: vals,
        }
    }
}

fn store_metadata<W: Pod>(dst: &mut [MaybeUninit<W>], meta_words: &[usize]) {
    let n_bytes = core::mem::size_of_val(meta_words);
    unsafe {
        ptr::copy(
            meta_words.as_ptr() as *const u8,
            dst.as_mut_ptr() as *mut u8,
            n_bytes,
        );
    }
}

fn read_metadata(src: *const usize, len: usize) -> [usize; 4] {
    unsafe {
        let mut meta = [0usize; 4];
        ptr::copy_nonoverlapping(src, meta.as_mut_ptr(), len);
        meta
    }
}

fn make_fat_ptr<T: ?Sized, W: Pod>(data: *mut (), meta: &[MaybeUninit<W>]) -> *mut T {
    unsafe {

        #[repr(C)]
        #[derive(Copy, Clone)]
        struct Raw {
            ptr: *const (),
            meta: [usize; 4],
        }

        union Inner<T: ?Sized> {
            ptr: *mut T,
            raw: Raw,
        }

        let mut rv = Inner {
            raw: Raw {
                ptr: data,
                meta: [0; 4],
            },
        };
        
        
        ptr::copy(
            meta.as_ptr() as *const u8,
            rv.raw.meta.as_mut_ptr() as *mut u8,
            meta.len() * mem::size_of::<W>(),
        );

        rv.ptr
    }
}

impl<T: ?Sized, D: DataBuf> Deref for Stacked<T, D> {
    type Target = T;
    
    fn deref(&self) -> &T {
        unsafe { &*self.as_ptr() }
    }
}

impl<T: ?Sized, D: DataBuf> DerefMut for Stacked<T, D> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.as_ptr_mut() }
    }
}

impl<T: ?Sized, D: DataBuf> Drop for Stacked<T, D> {
    fn drop(&mut self) {
        if mem::needs_drop::<T>() {
            unsafe {
                ptr::drop_in_place(self.as_ptr_mut());
            }
        }
    }
}

const WORD_SIZE: usize = mem::size_of::<usize>();

impl<T: ?Sized + Clone, D: DataBuf> Clone for Stacked<T, D>
where
    T: Clone,
    D: Default,
{
    fn clone(&self) -> Self {
        let mut data = D::default();
        unsafe {
            let src = self.as_ptr() as *const u8;
            let size = mem::size_of_val(&**self);
            let info_size = mem::size_of::<*mut T>() / WORD_SIZE - 1;
            let req_words = D::round_to_words(info_size * WORD_SIZE) + D::round_to_words(size);
            data.extend(req_words).unwrap();
            
            let dst = data.as_mut().as_mut_ptr() as *mut u8;
            ptr::copy_nonoverlapping(src, dst, size);
            
            let info_ofs = data.as_ref().len() - D::round_to_words(info_size * WORD_SIZE);
            let meta_ptr = data.as_mut()[info_ofs..].as_mut_ptr() as *mut usize;
            let meta = read_metadata(
                self.data.as_ref()[info_ofs..].as_ptr() as *const usize,
                info_size,
            );
            ptr::copy_nonoverlapping(meta.as_ptr(), meta_ptr, info_size);
        }
        
        Stacked {
            data,
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized + PartialEq, D: DataBuf> PartialEq for Stacked<T, D> {
    fn eq(&self, other: &Self) -> bool {
        self.deref().eq(other.deref())
    }
}

impl<T: ?Sized + fmt::Debug, D: DataBuf> fmt::Debug for Stacked<T, D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}

impl<T: ?Sized, D: DataBuf> AsRef<T> for Stacked<T, D> {
    fn as_ref(&self) -> &T {
        self.deref()
    }
}

impl<T: ?Sized, D: DataBuf> AsMut<T> for Stacked<T, D> {
    fn as_mut(&mut self) -> &mut T {
        self.deref_mut()
    }
}