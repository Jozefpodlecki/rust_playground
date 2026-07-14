use core::{mem, ptr};

use toolkit::println;

use crate::types::*;

#[derive(Clone, Copy)]
pub struct FnHandle {
    pub func_ptr: FuncPtr,
    pub data_ptr: DataPtr,
}

impl FnHandle {
    pub fn new(func_ptr: FuncPtr, data_ptr: DataPtr) -> Self {
        Self {
            func_ptr,
            data_ptr,
        }
    }

    pub fn data_size(&self) -> usize {
        unsafe {
            let header_ptr = (self.data_ptr.0 as usize - mem::size_of::<ClosureHeader>()) as *mut ClosureHeader;
            (*header_ptr).meta.data_size
        }
    }

    pub fn call<T>(&self) -> T {
        unsafe {
            let func: fn(*mut u8) -> T = mem::transmute(self.func_ptr.0);
            func(self.data_ptr.0)
        }
    }
}

pub struct HandlesIter {
    pub current: *mut ClosureHeader,
}

impl Iterator for HandlesIter {
    type Item = FnHandle;
    
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.current.is_null() {
                return None;
            }
            
            let meta = &(*self.current).meta;
            let handle = FnHandle::new(meta.func_ptr, meta.data_ptr);
            self.current = (*self.current).next;
            Some(handle)
        }
    }
}

pub struct StackedFunction<const N: usize> {
    pub data: [u8; N],
    pub func_ptr: *const u8,
    pub drop_fn: unsafe fn(*mut u8),
    pub size: usize,
}

impl<const N: usize> StackedFunction<N> {
    pub fn new() -> Self {
        Self {
            data: [0; N],
            func_ptr: ptr::null(),
            drop_fn: drop_nothing,
            size: 0,
        }
    }

    pub fn call<T>(&self) -> T {
        unsafe {
            let func: fn(*mut u8) -> T = mem::transmute(self.func_ptr);
            func(self.data.as_ptr() as *mut u8)
        }
    }
}

impl<const N: usize> Drop for StackedFunction<N> {
    fn drop(&mut self) {
        if !self.func_ptr.is_null() && self.size > 0 {
            unsafe {
                println!(">>> StackedFunction dropping closure data");
                (self.drop_fn)(self.data.as_mut_ptr());
            }
        }
    }
}

unsafe fn drop_nothing(_ptr: *mut u8) {}