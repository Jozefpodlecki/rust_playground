use core::cell::UnsafeCell;
use core::mem;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicU8, Ordering};

use crate::futex::{wait_on_address, wake_by_address_single};

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

pub struct Mutex<T> {
    state: AtomicMutexState,
    data: UnsafeCell<T>,
}

unsafe impl<T> Sync for Mutex<T> where T: Send {}

impl<T> Mutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            state: AtomicMutexState::new(MutexState::Unlocked),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        if !self.state.try_acquire() {
            self.lock_contended();
        }
        MutexGuard(self)
    }

    #[cold]
    fn lock_contended(&self) {
        let mut state = self.spin();

        if state == MutexState::Unlocked {
            if self.state.try_acquire() {
                return;
            }
            state = self.state.load();
        }

        loop {
            if state != MutexState::Contended && self.state.mark_contended() {
                return;
            }

            wait_on_address(&self.state.0, MutexState::Contended as u8, None);

            state = self.spin();
        }
    }

     fn spin(&self) -> MutexState {
        let mut spin = 100;
        loop {
            let state = self.state.load();

            if state != MutexState::Locked || spin == 0 {
                return state;
            }

            core::hint::spin_loop();
            spin -= 1;
        }
    }

    pub unsafe fn unlock(&self) {
        if self.state.unlock() {
            wake_by_address_single(&self.state.0);
        }
    }
}

pub struct MutexGuard<'a, T>(&'a Mutex<T>);

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        unsafe { self.0.unlock() }
    }
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.0.data.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.0.data.get() }
    }
}