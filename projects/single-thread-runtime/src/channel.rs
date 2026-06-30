use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};
use core::mem::MaybeUninit;
use alloc::boxed::Box;
use core::task::Waker;

pub struct Sender<T> {
    inner: *mut Channel<T>,
}

pub struct Receiver<T> {
    inner: *mut Channel<T>,
}

struct Channel<T> {
    ready: AtomicBool,
    value: UnsafeCell<MaybeUninit<T>>,
    waker: UnsafeCell<Option<Waker>>,
}

impl<T> Channel<T> {
    fn new() -> Self {
        Self {
            ready: AtomicBool::new(false),
            value: UnsafeCell::new(MaybeUninit::uninit()),
            waker: UnsafeCell::new(None),
        }
    }
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let channel = Box::new(Channel::new());
    let ptr = Box::into_raw(channel);
    (Sender { inner: ptr }, Receiver { inner: ptr })
}

unsafe impl<T: Send> Send for Sender<T> {}
unsafe impl<T: Send> Send for Receiver<T> {}

impl<T> Sender<T> {
    pub fn send(self, value: T) {
        unsafe {
            let channel = &mut *self.inner;
            (*channel.value.get()).write(value);
            channel.ready.store(true, Ordering::Release);
            
            if let Some(waker) = (*channel.waker.get()).take() {
                waker.wake();
            }
        }
    }
}

impl<T> Receiver<T> {
    pub fn try_recv(&self) -> Option<T> {
        unsafe {
            let channel = &*self.inner;
            if channel.ready.load(Ordering::Acquire) {
                let value = (*channel.value.get()).assume_init_read();
                Some(value)
            } else {
                None
            }
        }
    }

    pub fn register_waker(&self, waker: &Waker) {
        unsafe {
            let channel = &*self.inner;
            (*channel.waker.get()) = Some(waker.clone());
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.inner));
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {}
}