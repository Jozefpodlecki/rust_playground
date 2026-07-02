use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicUsize, Ordering};
use core::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use core::ptr;
use ntapi::ntobapi::NtWaitForSingleObject;
use toolkit::println;
use winapi::shared::ntdef::HANDLE;
use winapi::um::synchapi::{WaitOnAddress, WakeByAddressAll, WakeByAddressSingle};
use winapi::um::winnt::LARGE_INTEGER;

const MASK: usize = (1 << 30) - 1;
const WRITE_LOCKED: usize = MASK;
const READ_LOCKED: usize = 1;
const READERS_WAITING: usize = 1 << 30;
const WRITERS_WAITING: usize = 1 << 31;
const MAX_READERS: usize = MASK - 1;

#[repr(transparent)]
pub struct AtomicState(AtomicUsize);

impl AtomicState {
    pub const fn new() -> Self {
        Self(AtomicUsize::new(0))
    }

    pub fn load(&self) -> usize {
        self.0.load(Acquire)
    }

    pub fn store(&self, state: usize) {
        self.0.store(state, Release)
    }

    pub fn compare_exchange(&self, current: usize, new: usize) -> Result<usize, usize> {
        self.0.compare_exchange(current, new, Acquire, Relaxed)
    }

    pub fn fetch_add(&self, val: usize) -> usize {
        self.0.fetch_add(val, Release)
    }

    pub fn fetch_sub(&self, val: usize) -> usize {
        self.0.fetch_sub(val, Release)
    }

    pub fn try_acquire_read(&self) -> bool {
        let state = self.load();
        if is_read_lockable(state) {
            self.compare_exchange(state, state + READ_LOCKED).is_ok()
        } else {
            false
        }
    }

    pub fn try_acquire_write(&self) -> bool {
        self.compare_exchange(0, WRITE_LOCKED).is_ok()
    }

    pub fn release_read(&self) -> usize {
        self.fetch_sub(READ_LOCKED)
    }

    pub fn release_write(&self) -> usize {
        self.fetch_sub(WRITE_LOCKED)
    }

    pub fn spin_until(&self, f: impl Fn(usize) -> bool) -> usize {
        let mut spin = 100;
        loop {
            let state = self.0.load(Relaxed);
            if f(state) || spin == 0 {
                return state;
            }
            core::hint::spin_loop();
            spin -= 1;
        }
    }

    pub fn spin_write(&self) -> usize {
        self.spin_until(|s| is_unlocked(s) || s & WRITERS_WAITING != 0)
    }

    pub fn spin_read(&self) -> usize {
        self.spin_until(|s| !is_write_locked(s) || s & READERS_WAITING != 0 || s & WRITERS_WAITING != 0)
    }
}

#[inline]
fn is_unlocked(state: usize) -> bool {
    state & MASK == 0
}

#[inline]
fn is_write_locked(state: usize) -> bool {
    state & MASK == WRITE_LOCKED
}

#[inline]
fn is_read_lockable(state: usize) -> bool {
    state & MASK < MAX_READERS && state & READERS_WAITING == 0 && state & WRITERS_WAITING == 0
}

pub struct RwLock<T> {
    state: AtomicState,
    writer_notify: AtomicUsize,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send + Sync> Send for RwLock<T> {}
unsafe impl<T: Send + Sync> Sync for RwLock<T> {}

impl<T> RwLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            state: AtomicState::new(),
            writer_notify: AtomicUsize::new(0),
            data: UnsafeCell::new(data),
        }
    }

    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        if !self.state.try_acquire_read() {
            self.read_contended();
        }
        RwLockReadGuard(self)
    }

    pub fn try_read(&self) -> Option<RwLockReadGuard<'_, T>> {
        if self.state.try_acquire_read() {
            Some(RwLockReadGuard(self))
        } else {
            None
        }
    }

    pub fn write(&self) -> RwLockWriteGuard<'_, T> {
        if !self.state.try_acquire_write() {
            self.write_contended();
        }
        RwLockWriteGuard(self)
    }

    pub fn try_write(&self) -> Option<RwLockWriteGuard<'_, T>> {
        if self.state.try_acquire_write() {
            Some(RwLockWriteGuard(self))
        } else {
            None
        }
    }

    #[cold]
    fn read_contended(&self) {
        let mut has_slept = false;
        let mut state = self.state.spin_read();

        loop {
            if (has_slept && is_read_lockable_after_downgrade(state)) || is_read_lockable(state) {
                if self.state.compare_exchange(state, state + READ_LOCKED).is_ok() {
                    return;
                }
                state = self.state.load();
                continue;
            }

            assert!(state & MASK < MAX_READERS, "too many active read locks");

            if state & READERS_WAITING == 0 {
                if self.state.compare_exchange(state, state | READERS_WAITING).is_err() {
                    state = self.state.load();
                    continue;
                }
            }

            self.wait_on_address(&self.state.0, state | READERS_WAITING);
            has_slept = true;
            state = self.state.spin_read();
        }
    }

    #[cold]
    fn write_contended(&self) {
        let mut state = self.state.spin_write();
        let mut other_writers_waiting = 0;

        loop {
            if is_unlocked(state) {
                if self.state
                    .compare_exchange(state, state | WRITE_LOCKED | other_writers_waiting)
                    .is_ok()
                {
                    return;
                }
                state = self.state.load();
                continue;
            }

            if state & WRITERS_WAITING == 0 {
                if self.state.compare_exchange(state, state | WRITERS_WAITING).is_err() {
                    state = self.state.load();
                    continue;
                }
            }

            other_writers_waiting = WRITERS_WAITING;
            let seq = self.writer_notify.load(Acquire);
            state = self.state.load();

            if is_unlocked(state) || state & WRITERS_WAITING == 0 {
                continue;
            }

            self.wait_on_address(&self.writer_notify, seq);
            state = self.state.spin_write();
        }
    }

    fn wake_writer_or_readers(&self, state: usize) {
        if state == WRITERS_WAITING {
            if self.state.compare_exchange(state, 0).is_ok() {
                self.wake_writer();
                return;
            }
        }

        if state == READERS_WAITING + WRITERS_WAITING {
            if self.state.compare_exchange(state, READERS_WAITING).is_ok() {
                self.wake_writer();
                return;
            }
        }

        if state == READERS_WAITING {
            if self.state.compare_exchange(state, 0).is_ok() {
                self.wake_readers();
            }
        }
    }

    fn wake_writer(&self) {
        self.writer_notify.fetch_add(1, Release);
        self.wake_by_address_single(&self.writer_notify);
    }

    fn wake_readers(&self) {
        self.wake_by_address_all(&self.state.0);
    }

    fn wait_on_address(&self, addr: &AtomicUsize, expected: usize) {
        unsafe {
            // let timeout = 0i64;
            // let status = NtWaitForSingleObject(
            //     addr as *const _ as *mut _,
            //     0,
            //     ptr::null_mut(),
            // );
            let result = WaitOnAddress(
                addr as *const _ as *mut _,
                &expected as *const _ as *mut _,
                core::mem::size_of::<usize>(),
                winapi::um::winbase::INFINITE,
            );
        }
    }

    fn wake_by_address_single(&self, addr: &AtomicUsize) {
        unsafe {
            WakeByAddressSingle(addr as *const _ as *mut _);            
        }
    }

    fn wake_by_address_all(&self, addr: &AtomicUsize) {
        unsafe {
            WakeByAddressAll(addr as *const _ as *mut _);
        }
    }
}

#[inline]
fn is_read_lockable_after_downgrade(state: usize) -> bool {
    state & MASK < MAX_READERS
        && state & READERS_WAITING == 0
        && !is_write_locked(state)
        && !is_unlocked(state)
}

pub struct RwLockReadGuard<'a, T>(&'a RwLock<T>);

impl<T> Drop for RwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        let state = self.0.state.release_read() - READ_LOCKED;
        if is_unlocked(state) && state & WRITERS_WAITING != 0 {
            self.0.wake_writer_or_readers(state);
        }
    }
}

impl<T> Deref for RwLockReadGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.0.data.get() }
    }
}

pub struct RwLockWriteGuard<'a, T>(&'a RwLock<T>);

impl<T> Drop for RwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        let state = self.0.state.release_write() - WRITE_LOCKED;
        if state & WRITERS_WAITING != 0 || state & READERS_WAITING != 0 {
            self.0.wake_writer_or_readers(state);
        }
    }
}

impl<T> Deref for RwLockWriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.0.data.get() }
    }
}

impl<T> DerefMut for RwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.0.data.get() }
    }
}