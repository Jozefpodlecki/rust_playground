#![no_std]
#![allow(unsafe_op_in_unsafe_fn)]
#![feature(core_intrinsics)]

mod x86_64;

#[inline(never)]
#[unsafe(no_mangle)]
pub extern "C" fn __CxxFrameHandler3() {
    
}

#[inline(never)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    x86_64::copy_forward(dest, src, n);
    dest
}

#[inline(never)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(dest: *mut u8, c: i32, n: usize) -> *mut u8 {
    x86_64::set_bytes(dest, c as u8, n);
    dest
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let delta = (dest as usize).wrapping_sub(src as usize);
    if delta >= n {
        x86_64::copy_forward(dest, src, n);
    } else {
        x86_64::copy_backward(dest, src, n);
    }
    dest
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    x86_64::compare_bytes(s1, s2, n)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strlen(s: *const core::ffi::c_char) -> usize {
    x86_64::c_string_length(s)
}