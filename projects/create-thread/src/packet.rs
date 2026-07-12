use core::ptr;

use toolkit::{Mutex, MutexGuard};

use crate::arena::{ArenaPtr};

pub struct Packet<T>(pub(crate) Mutex<PacketInner<T>>);

pub(crate) struct PacketInner<T> {
    result: Option<T>,
    finished: bool,
    pub(crate) closure_ptr: ArenaPtr,
}

impl<T> Packet<T> {
    pub fn new(closure_ptr: ArenaPtr) -> Self {
        Self(Mutex::new(PacketInner {
            result: None,
            finished: false,
            closure_ptr,
        }))
    }

    pub fn from_ptr(ptr: *mut winapi::ctypes::c_void) -> &'static mut Self {
        unsafe { &mut *(ptr as *mut Self) }
    }

    pub fn take_closure<F>(&self) -> F
    where
        F: FnOnce() -> T + 'static,
    {
        let mut guard = self.0.lock();
        let closure_ptr = guard.closure_ptr;
        unsafe { ptr::read(closure_ptr.0 as *const F) } 
    }

    pub fn write_to(self: *mut Packet<T>, closure_ptr: ArenaPtr) {
        unsafe { ptr::write(self, Self::new(closure_ptr)); }
    }

    pub fn store_result(&self, result: T) {
        let mut guard = self.0.lock();
        guard.result = Some(result);
        guard.finished = true;
    }

    pub fn take_result(&self) -> Option<T> {
        let mut guard = self.0.lock();
        guard.result.take()
    }

    pub fn is_finished(&self) -> bool {
        self.0.lock().finished
    }
}