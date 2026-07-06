use core::sync::atomic::{Atomic, AtomicBool, AtomicUsize};
use core::sync::{atomic::{AtomicPtr, Ordering}};
use core::time::Duration;
use core::sync::atomic::*;
use core::{ffi::c_void, sync::atomic::AtomicU32};

use winapi::um::synchapi::{WaitOnAddress, WakeByAddressSingle};

pub unsafe trait Waitable {
    type Futex;
}

pub unsafe trait Futexable {}

pub fn wait_on_address<W: Waitable>(
    address: &W::Futex,
    compare: W,
    timeout: Option<Duration>,
) -> bool {
    unsafe {
        let addr = core::ptr::from_ref(address).cast::<c_void>();
        let size = size_of::<W>();
        let compare_addr = (&raw const compare).cast::<c_void>();
        let timeout = timeout.map(dur2timeout).unwrap_or(INFINITE);
        WaitOnAddress(addr as _, compare_addr as _, size, timeout) == 1
    }
}

pub const INFINITE: u32 = 4294967295u32;

macro_rules! unsafe_waitable_int {
    ($(($int:ty, $atomic:ty)),*$(,)?) => {
        $(
            unsafe impl Waitable for $int {
                type Futex = $atomic;
            }
            unsafe impl Futexable for $atomic {}
        )*
    };
}
unsafe_waitable_int! {
    (bool, AtomicBool),
    (i8, AtomicI8),
    (i16, AtomicI16),
    (i32, AtomicI32),
    (i64, AtomicI64),
    (isize, AtomicIsize),
    (u8, AtomicU8),
    (u16, AtomicU16),
    (u32, AtomicU32),
    (u64, AtomicU64),
    (usize, AtomicUsize),
}
unsafe impl<T> Waitable for *const T {
    type Futex = Atomic<*mut T>;
}
unsafe impl<T> Waitable for *mut T {
    type Futex = Atomic<*mut T>;
}

unsafe impl<T> Futexable for AtomicPtr<T> {}

pub fn dur2timeout(dur: Duration) -> u32 {
    dur.as_secs()
        .checked_mul(1000)
        .and_then(|ms| ms.checked_add((dur.subsec_nanos() as u64) / 1_000_000))
        .and_then(|ms| ms.checked_add(if dur.subsec_nanos() % 1_000_000 > 0 { 1 } else { 0 }))
        .map(|ms| if ms > <u32>::MAX as u64 { INFINITE } else { ms as u32 })
        .unwrap_or(INFINITE)
}

pub fn wake_by_address_single<T: Futexable>(address: &T) {
    unsafe {
        let addr = core::ptr::from_ref(address).cast::<c_void>();
        WakeByAddressSingle(addr as _);
    }
}