use ntapi::ntpsapi::{NtCurrentProcess, THREAD_BASIC_INFORMATION, ThreadBasicInformation};
use toolkit::{println, syscalls::{NtAllocateVirtualMemory, NtDelayExecution}};
use winapi::um::winnt::LARGE_INTEGER;
use toolkit::{SystemError, syscalls::*};
use winapi::{shared::{ntdef::HANDLE, ntstatus::STATUS_SUCCESS}, um::winnt::{MEM_COMMIT, MEM_RESERVE, PAGE_READWRITE}};


pub fn delay_infinite(alertable: u8) {
    delay(alertable, i64::MIN)
}

pub fn delay_secs(alertable: u8, value: i64) {
    delay(alertable, value * -10_000_000)
}

pub fn delay(alertable: u8, value: i64) {
    unsafe {
        let mut delay: LARGE_INTEGER = core::mem::zeroed();
        *delay.QuadPart_mut() = value;
        NtDelayExecution(alertable, &mut delay);
    }
}

pub fn verify_stack_copy(src: *mut u8, dst: *mut u8, size: usize) -> bool {
    let chunk_size = 64;
    let mut offset = 0;
    let mut all_equal = true;

    while offset < size {
        let src_ptr = unsafe { src.add(offset) };
        let dst_ptr = unsafe { dst.add(offset) };
        
        for i in 0..chunk_size {
            if offset + i >= size {
                break;
            }
            unsafe {
                if *src_ptr.add(i) != *dst_ptr.add(i) {
                    println!("Mismatch at offset 0x{:X}: src=0x{:02X} dst=0x{:02X}", 
                             offset + i, *src_ptr.add(i), *dst_ptr.add(i));
                    all_equal = false;
                    return false;
                }
            }
        }
        
        offset += chunk_size;
    }

    if all_equal {
        println!("Stack copy verified: {} bytes match", size);
    }
    
    all_equal
}

pub struct StackCopy {
    pub new_base: usize,
    pub new_stack_limit: usize,
    pub original_stack_base: usize,
    pub original_stack_limit: usize,
    pub stack_size: usize,
}

pub fn copy_thread_stack(thread_handle: HANDLE) -> Result<StackCopy, SystemError> {
    let mut basic_info: THREAD_BASIC_INFORMATION = unsafe { core::mem::zeroed() };
    let mut return_length: u32 = 0;

    let status = unsafe {
        NtQueryInformationThread(
            thread_handle,
            ThreadBasicInformation,
            &mut basic_info as *mut _ as *mut _,
            core::mem::size_of::<THREAD_BASIC_INFORMATION>() as u32,
            &mut return_length,
        )
    };

    if status != STATUS_SUCCESS {
        return Err(SystemError::NtStatus(status));
    }

    unsafe {
        let teb_ptr = basic_info.TebBaseAddress;
        let teb = &*teb_ptr;

        let stack_limit = teb.NtTib.StackLimit;
        let stack_base = teb.NtTib.StackBase;
        let stack_size = (stack_base as usize) - (stack_limit as usize);

        let mut target_stack = core::ptr::null_mut();
        let mut region_size = stack_size;

        let status = NtAllocateVirtualMemory(
            NtCurrentProcess,
            &mut target_stack,
            0,
            &mut region_size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        );

        if status != STATUS_SUCCESS {
            return Err(SystemError::NtStatus(status));
        }

        let mut bytes_read: usize = 0;
        let status = NtReadVirtualMemory(
            NtCurrentProcess,
            stack_limit as *mut _,
            target_stack,
            stack_size,
            &mut bytes_read,
        );

        if status != STATUS_SUCCESS {
            return Err(SystemError::NtStatus(status));
        }

        Ok(StackCopy {
            new_base: target_stack as usize,
            new_stack_limit: target_stack as usize + stack_size,
            original_stack_base: stack_base as usize,
            original_stack_limit: stack_limit as usize,
            stack_size,
        })
    }
}

