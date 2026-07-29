#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![allow(static_mut_refs)]

mod utils;
mod diag;
mod error;
mod event;
mod win;
mod types;
mod nt;

use core::{panic::PanicInfo, time::Duration};

use toolkit::{ProcessMemoryReader, ProcessQuerier, ProcessSpawner, Sleeper, U8CStackString, U16CStackString, println, system_threads};

use crate::{error::DebuggerError, event::{DebugEventImpl, ExitProcessEvent, ExitThreadEvent}, nt::*, types::*, win::WinDebugger};

extern crate builtins;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {
    let file_path = U16CStackString::<150>::from_str(
        r#"C:\repos\rust_playground\projects\test-debug\target\debug\test-debug.exe"#
    ).unwrap();
    let process_name = U8CStackString::<50>::from_str(r#"test-debug.exe"#).unwrap();
    
    let pid = if let Some(pid) = ProcessQuerier::find_process_by_name(&process_name) {
        pid
    } else {
        let info = ProcessSpawner::create_suspended(file_path).unwrap();
        info.pid
    };

    

    let options = NativeWinDebuggerOptions {
        store_debug_object_in_teb: true
    };
    // let mut debugger = WinDebugger::new(win::WinDebuggerOptions::default());
    let mut debugger = NativeWinDebugger::new(options);
    // println!("ProcessSpawner tid: {}", tid);

    if let Err(err) = debugger.attach(pid) {
        println!("[!] Failed to attach debugger to process! {err}");
        return 1;
    }

    let process_handle = debugger.handle();
    let peb_ptr = ProcessQuerier::query_peb(process_handle).unwrap();
    let being_debugged_address = peb_ptr as usize + 0x2;
    let mut being_debugged: u8 = 0;
    
    let bytes_read = ProcessMemoryReader::read_remote::<u8>(
        process_handle,
        being_debugged_address as *mut _
    ).unwrap();

    println!("{bytes_read}");

    if let Err(err) = debugger.resume_main_thread() {
        println!("[!] Failed to resume thread! {err}");
    }

    loop {
        let event = match debugger.wait_for_event(WaitOptions::with_timeout(Duration::from_secs(1))) {
            Ok(event) => event,
            Err(DebuggerError::Timeout) => continue,
            Err(err) => {
                println!("{err}");
                break;
            },
        };

        println!("name={} thread_id={}", event.name(), event.thread_id());
        let tid = event.thread_id();

        if let Err(err) = debugger.continue_event(tid, ContinueStatus::Continue) {
            println!("continue_event failed {err}");
            break;
        }

        if let Some(event) = event.as_event::<ExitProcessEvent>() {
            break;
        }

        Sleeper::sleep(100);
    }

    0
}