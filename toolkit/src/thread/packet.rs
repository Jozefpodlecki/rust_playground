use core::cell::UnsafeCell;
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};
use alloc::boxed::Box;
use ntapi::ntobapi::{NtClose, NtWaitForSingleObject};
use ntapi::ntpsapi::NtCurrentProcess;
use winapi::shared::ntdef::{NTSTATUS, HANDLE};
use winapi::shared::ntstatus::STATUS_SUCCESS;
use winapi::um::winnt::{THREAD_ALL_ACCESS, LARGE_INTEGER};
use crate::arc::Arc;

struct PacketInner<T> {
    finished: AtomicBool,
    result: UnsafeCell<Option<T>>,
    func: UnsafeCell<Option<Box<dyn FnOnce() -> T + Send>>>,
}

pub struct Packet<T>(Arc<PacketInner<T>>);

impl<T> Clone for Packet<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Packet<T> {
    pub fn new(func: impl FnOnce() -> T + Send + 'static) -> Self {
        Self(Arc::new(PacketInner {
            finished: AtomicBool::new(false),
            result: UnsafeCell::new(None),
            func: UnsafeCell::new(Some(Box::new(func))),
        }))
    }

    pub fn from_ptr(ptr: *mut Self) -> Self {
        unsafe { ptr.read() }
    }

    pub fn execute(&self) -> T {
        unsafe {
            let inner = &*self.0;
            let func_ptr = &mut *inner.func.get();
            let func = func_ptr.take().unwrap();
            func()
        }
    }

    pub fn set_result(&self, result: T) {
        unsafe {
            let inner = &*self.0;
            inner.result.get().write(Some(result));
            inner.finished.store(true, Ordering::Release);
        }
    }

    pub fn is_finished(&self) -> bool {
        (*self.0).finished.load(Ordering::Acquire)
    }

    pub fn take_result(&self) -> Option<T> {
        unsafe {
            let inner = &*self.0;
            (*inner.result.get()).take()
        }
    }

    pub fn drop(self: *mut Self) {
        unsafe { let _ = Box::from_raw(self); }
    }

    pub fn leak(&self) -> *mut Self {
        let leaked: &mut Self = Box::leak(Box::new(self.clone()));
        leaked as *mut Self
    }
}