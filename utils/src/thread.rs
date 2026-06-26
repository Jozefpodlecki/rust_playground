use core::cell::UnsafeCell;
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};
use ntapi::ntobapi::NtClose;
use ntapi::ntobapi::NtWaitForSingleObject;
use ntapi::ntpsapi::NtCreateThreadEx;
use ntapi::ntpsapi::NtCurrentProcess;
use winapi::shared::ntdef::HANDLE;
use winapi::shared::ntdef::NTSTATUS;
use winapi::shared::ntstatus::STATUS_SUCCESS;
use winapi::um::winnt::LARGE_INTEGER;
use winapi::um::winnt::THREAD_ALL_ACCESS;
use crate::arc::Arc;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

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

    pub fn leak(&self) -> *mut Self {
        let leaked: &mut Self = Box::leak(Box::new(self.clone()));
        leaked as *mut Self
    }
}

pub struct JoinHandle<T> {
    handle: HANDLE,
    packet: Packet<T>,
}

impl<T> JoinHandle<T> {
    pub fn is_finished(&self) -> bool {
        
        let status = unsafe {
            let mut delay: LARGE_INTEGER = core::mem::zeroed();
            *delay.QuadPart_mut() = 0;
            NtWaitForSingleObject(
                self.handle,
                0,
                &mut delay,
            )
        };
        status == STATUS_SUCCESS
    }

    pub fn join(self) -> Result<T, ()> {
        unsafe {
            let status = NtWaitForSingleObject(
                self.handle,
                0,
                ptr::null_mut(),
            );
            
            if status != STATUS_SUCCESS {
                return Err(());
            }
            
            NtClose(self.handle);
            
            self.packet.take_result().ok_or(())
        }
    }
}

pub struct Thread;

impl Thread {

    pub fn spawn_ex<F, T>(func: F) -> Result<JoinHandle<T>, NTSTATUS>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        let packet = Packet::new(func);
        
        unsafe extern "system" fn thread_entry<T>(param: *mut winapi::ctypes::c_void) -> u32 {
            let packet = Packet::<T>::from_ptr(param as _);
            let result = packet.execute();
            packet.set_result(result);
            
            0
        }

        let packet_clone = packet.leak();
        let param_ptr: *mut winapi::ctypes::c_void = packet_clone as _;
        let mut thread_handle: HANDLE = ptr::null_mut();

        let status = unsafe {
            NtCreateThreadEx(
                &mut thread_handle,
                THREAD_ALL_ACCESS,
                ptr::null_mut(),
                NtCurrentProcess,
                thread_entry::<T> as *mut _,
                param_ptr,
                0,
                0,
                0,
                0,
                ptr::null_mut(),
            )
        };

        if status != STATUS_SUCCESS {
            return Err(status);
        }

        Ok(JoinHandle {
            handle: thread_handle,
            packet,
        })
    }
}