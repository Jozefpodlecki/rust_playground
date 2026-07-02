use core::ops::{Deref, DerefMut};
use core::ptr;
use core::mem;
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, AtomicU8, Ordering};
use alloc::boxed::Box;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutexState {
    Unlocked = 0,
    Locked = 1,
    Contended = 2,
}

#[repr(transparent)]
pub struct AtomicMutexState(AtomicU8);

impl AtomicMutexState {
    pub const fn new(state: MutexState) -> Self {
        Self(AtomicU8::new(state as u8))
    }

    pub fn load(&self) -> MutexState {
        unsafe { mem::transmute(self.0.load(Ordering::Acquire)) }
    }

    pub fn try_acquire(&self) -> bool {
        self.0
            .compare_exchange(
                MutexState::Unlocked as u8,
                MutexState::Locked as u8,
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .is_ok()
    }

    pub fn mark_contended(&self) -> bool {
        self.0.swap(MutexState::Contended as u8, Ordering::Acquire) == MutexState::Unlocked as u8
    }

    pub fn unlock(&self) -> bool {
        self.0.swap(MutexState::Unlocked as u8, Ordering::Release) == MutexState::Contended as u8
    }
}

struct ArcInnerMutex<T: ?Sized> {
    count: AtomicUsize,
    state: AtomicMutexState,
    data: UnsafeCell<T>,
}

pub struct ArcMutex<T: ?Sized>(*mut ArcInnerMutex<T>);

unsafe impl<T: Send + Sync + ?Sized> Send for ArcMutex<T> {}
unsafe impl<T: Send + Sync + ?Sized> Sync for ArcMutex<T> {}

impl<T> ArcMutex<T> {
    pub fn new(data: T) -> Self {
        let inner = Box::new(ArcInnerMutex {
            count: AtomicUsize::new(1),
            state: AtomicMutexState::new(MutexState::Unlocked),
            data: UnsafeCell::new(data),
        });
        Self(Box::into_raw(inner))
    }
}

impl<T: ?Sized> ArcMutex<T> {
    pub fn clone(&self) -> Self {
        unsafe {
            (*self.0).count.fetch_add(1, Ordering::Relaxed);
        }
        Self(self.0)
    }

    pub fn lock(&self) -> ArcMutexGuard<'_, T> {
        unsafe {
            if !(*self.0).state.try_acquire() {
                self.lock_contended();
            }
            ArcMutexGuard {
                inner: self.0,
                _marker: core::marker::PhantomData,
            }
        }
    }

    #[cold]
    fn lock_contended(&self) {
        let mut spin_count = 100;
        unsafe {
            loop {
                match (*self.0).state.load() {
                    MutexState::Unlocked => {
                        if (*self.0).state.try_acquire() {
                            return;
                        }
                    }
                    MutexState::Locked => {
                        if spin_count == 0 {
                            if (*self.0).state.mark_contended() {
                                return;
                            }
                        }
                        spin_count -= 1;
                        core::hint::spin_loop();

                        //    if spin_count == 0 {
                        //     // Mark as contended and wait
                        //     let old = (*self.0).state.0.swap(MutexState::Contended as u8, Ordering::Acquire);
                        //     if old == MutexState::Unlocked as u8 {
                        //         // It became unlocked while we marked, try acquire again
                        //         if (*self.0).state.try_acquire() {
                        //             return;
                        //         }
                        //     } else {
                        //         // Wait until state changes from Contended
                        //         let compare = MutexState::Contended as u8;
                        //         WaitOnAddress(
                        //             state_ptr,
                        //             &compare as *const _ as *const c_void,
                        //             mem::size_of::<u8>(),
                        //             u32::MAX, // INFINITE
                        //         );
                        //     }
                        //     spin_count = 100; // Reset spin count after wait
                        // } else {
                        //     spin_count -= 1;
                        //     core::hint::spin_loop();
                        // }
                    }
                    MutexState::Contended => {
                        core::hint::spin_loop();
                        // let compare = MutexState::Contended as u8;
                        // WaitOnAddress(
                        //     state_ptr,
                        //     &compare as *const _ as *const c_void,
                        //     mem::size_of::<u8>(),
                        //     u32::MAX, // INFINITE
                        // );
                    }
                }
            }
        }
    }

    pub unsafe fn unlock(&self) {
        unsafe {
            if (*self.0).state.unlock() {
                // futex_wake()
                // WakeByAddressSingle((&(*self.0).state.0) as *const _ as *const c_void);
            }
        }
    }

    pub fn strong_count(&self) -> usize {
        unsafe { (*self.0).count.load(Ordering::Relaxed) }
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.strong_count() == 1 {
            unsafe { Some(&mut *(*self.0).data.get()) }
        } else {
            None
        }
    }
}

impl<T: ?Sized> Drop for ArcMutex<T> {
    fn drop(&mut self) {
        unsafe {
            if (*self.0).count.fetch_sub(1, Ordering::Release) != 1 {
                return;
            }
            core::sync::atomic::fence(Ordering::Acquire);
            ptr::drop_in_place((*self.0).data.get());
            drop(Box::from_raw(self.0));
        }
    }
}

pub struct ArcMutexGuard<'a, T: ?Sized> {
    inner: *mut ArcInnerMutex<T>,
    _marker: core::marker::PhantomData<&'a mut T>,
}

impl<'a, T: ?Sized> Deref for ArcMutexGuard<'a, T> {
    type Target = T;
    
    fn deref(&self) -> &T {
        unsafe { &*((*self.inner).data.get()) }
    }
}

impl<'a, T: ?Sized> DerefMut for ArcMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *((*self.inner).data.get()) }
    }
}

impl<'a, T: ?Sized> Drop for ArcMutexGuard<'a, T> {
    fn drop(&mut self) {
        unsafe {
            if (*self.inner).state.unlock() {
                // futex_wake()
                //  WakeByAddressSingle((&(*self.0).state.0) as *const _ as *const c_void);
            }
        }
    }
}