use core::ptr::null_mut;

use heapless::Vec;
use iced_x86::*;
use winapi::shared::ntdef::NTSTATUS;

pub fn trampoline_to<const N: usize>(addr: usize) -> Result<Vec<u8, N>, IcedError> {
    let mut encoder = Encoder::new(64);
    let mut rip = 0;

    let mov_rax = Instruction::with2(Code::Mov_r64_imm64, Register::RAX, addr as u64)?;
    rip += encoder.encode(&mov_rax, rip as _)?;
    let jmp_rax = Instruction::with1(Code::Jmp_rm64, Register::RAX)?;
    rip += encoder.encode(&jmp_rax, rip as _)?;

    let ret = Instruction::with(Code::Retnq);
    rip += encoder.encode(&ret, rip as _)?;

    let buffer = encoder.take_buffer();
    let output = Vec::from_iter(buffer);

    Ok(output)
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