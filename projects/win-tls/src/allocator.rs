
use alloc::boxed::Box;
use ntapi::{ntpebteb::TEB, winapi_local::um::winnt::NtCurrentTeb};
use toolkit::println;
use winapi::shared::ntdef::PVOID;

pub struct ThreadLocalAllocator;

impl ThreadLocalAllocator {
    pub const TLS_SLOT_COUNT: usize = 64;

    pub fn thread_id() -> usize {
        let teb = unsafe { NtCurrentTeb() };
        unsafe { (*teb).ClientId.UniqueThread as _ }
    }

    pub fn alloc<T>(value: T) -> Option<usize> {
        unsafe {
            let teb = NtCurrentTeb();
            let slots = &mut (*teb).TlsSlots;
            
            for slot in 0..Self::TLS_SLOT_COUNT {
                if slots[slot].is_null() {
                    let boxed = Box::into_raw(Box::new(value));
                    slots[slot] = boxed as PVOID;
                    return Some(slot);
                }
            }
            
            None
        }
    }

    pub fn get<T>(slot: usize) -> Option<&'static T> {
        unsafe {
            if slot >= Self::TLS_SLOT_COUNT {
                return None;
            }
            
            let teb = NtCurrentTeb();
            let ptr = (*teb).TlsSlots[slot] as *const T;
            
            if ptr.is_null() {
                None
            } else {
                Some(&*ptr)
            }
        }
    }

    pub fn get_mut<T>(slot: usize) -> Option<&'static mut T> {
         unsafe {
            if slot >= Self::TLS_SLOT_COUNT {
                return None;
            }
            
            let teb = NtCurrentTeb();
            let ptr = (*teb).TlsSlots[slot] as *mut T;
            
            if ptr.is_null() {
                None
            } else {
                Some(&mut *ptr)
            }
        }
    }

    pub fn take<T>(slot: usize) -> Option<T> {
        unsafe {
            if slot >= Self::TLS_SLOT_COUNT {
                return None;
            }
            
            let teb = NtCurrentTeb();
            let ptr = (*teb).TlsSlots[slot] as *mut T;
            
            if ptr.is_null() {
                None
            } else {
                (*teb).TlsSlots[slot] = core::ptr::null_mut();
                Some(*Box::from_raw(ptr))
            }
        }
    }

    pub fn free(slot: usize) -> bool {
        unsafe {
            if slot >= Self::TLS_SLOT_COUNT {
                return false;
            }
            
            let teb = NtCurrentTeb();
            let ptr = (*teb).TlsSlots[slot];
            
            if ptr.is_null() {
                return false;
            }
            
            (*teb).TlsSlots[slot] = core::ptr::null_mut();
            drop(Box::from_raw(ptr as *mut u8));
            true
        }
    }
}