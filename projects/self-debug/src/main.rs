#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![allow(static_mut_refs)]

use core::panic::PanicInfo;

use ntapi::{ntpsapi::{NtCurrentProcess, NtCurrentProcessId}, ntrtl::RtlAddVectoredExceptionHandler};
use toolkit::{ProcessEnvironmentBlock, Sleeper, println, syscalls::NtTerminateProcess};
use winapi::{shared::ntdef::NTSTATUS, um::winnt::EXCEPTION_POINTERS, vc::excpt::EXCEPTION_CONTINUE_SEARCH};

use crate::utils::*;

extern crate builtins;

mod utils;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    unsafe { NtTerminateProcess(NtCurrentProcess, 0); }
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {

    if let Err(err) = run() {
        println!("err {err}");
        return err;
    }
    
    0
}

unsafe extern "system" fn handle_exception(
    exception_info: *mut EXCEPTION_POINTERS,
) -> i32 {
    println!("test");
    EXCEPTION_CONTINUE_SEARCH
}

fn run() -> Result<NTSTATUS, NTSTATUS> {
    let pid = unsafe { NtCurrentProcessId() as u32 };
    println!("pid: {pid}");
    let peb = ProcessEnvironmentBlock::current_process();
    let args = peb.command_line();
    unsafe { RtlAddVectoredExceptionHandler(0, Some(handle_exception)); }
    // enable_debug_privilege()?;

    if let Some(parent_pid) = args.at(1).and_then(|pr| pr.to_u32()) {
        let process_handle = debug_parent(parent_pid)?;

        loop {
            let status = check_status(process_handle);
            
            match status {
                ProcessStatus::Dead(exit_code) => {
                    return Ok(exit_code);
                },
                _ => {}
            }

            Sleeper::sleep(1000);
        }

        return Ok(0);
    }
    
    let process_handle = spawn_child(peb, pid)?;
    spawn_wait_thread(NtCurrentProcess, process_handle)?;

    loop {
        println!("sleep");
        Sleeper::sleep(1000);
    }

    Ok(0)
}