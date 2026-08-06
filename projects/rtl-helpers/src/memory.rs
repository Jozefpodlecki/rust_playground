use winapi::{ctypes::c_void, um::{winnt::*}};

pub struct Memory;

impl Memory {
    pub fn copy(dest: *mut c_void, src: *const c_void, len: usize) {
        unsafe { RtlCopyMemory(dest, src, len) }
    }

    pub fn move_memory(dest: *mut c_void, src: *const c_void, len: usize) {
        unsafe { RtlMoveMemory(dest, src, len) }
    }

    pub fn fill(dest: *mut c_void, len: usize, value: u8) {
        unsafe { RtlFillMemory(dest, len, value) }
    }

    pub fn zero(dest: *mut c_void, len: usize) {
        unsafe { RtlZeroMemory(dest, len) }
    }
}