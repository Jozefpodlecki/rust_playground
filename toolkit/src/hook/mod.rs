use core::{arch::asm, ptr::null_mut};

use heapless::Vec;
// use iced_x86::*;
use winapi::shared::ntdef::NTSTATUS;

use crate::encoder::{self, Encoder1K, EncoderError};

pub fn jmp_trampoline_to<const N: usize>(addr: usize) -> Result<Vec<u8, 1024>, EncoderError> {

    let mut encoder = Encoder1K::new();

    encoder.mov().rax().imm64(addr as _);
    encoder.jmp().rax();
    encoder.ret().near();

    Ok(encoder.into_buffer())
}

pub fn hook_function(
    handle: *mut winapi::ctypes::c_void,
    func_ptr: usize,
    hook_addr: usize,
    buffer: &[u8]
) -> Result<(), NTSTATUS> {
    unsafe {
        let mut page_base = func_ptr as *mut winapi::ctypes::c_void;
        let mut region_size = buffer.len();
        let mut old_protect = 0u32;

        let status = crate::syscalls::NtProtectVirtualMemory(
            handle,
            &mut page_base,
            &mut region_size,
            winapi::um::winnt::PAGE_EXECUTE_READWRITE,
            &mut old_protect,
        );

        if status < 0 {
            return Err(status);
        }
        
        let mut func_ptr = func_ptr as *mut winapi::ctypes::c_void;

        let status = crate::syscalls::NtWriteVirtualMemory(
            handle,
            func_ptr,
            buffer.as_ptr() as *mut _,
            buffer.len() as _,
            null_mut(),
        );

        if status < 0 {
            return Err(status);
        }

        region_size = buffer.len();
        let status = crate::syscalls::NtProtectVirtualMemory(
            handle,
            &mut func_ptr,
            &mut region_size,
            old_protect,
            &mut 0,
        );

        if status < 0 {
            return Err(status);
        }

        Ok(())
    }
}