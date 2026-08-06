#![no_std]
#![no_main]
#![windows_subsystem = "console"]

use core::{mem, panic::PanicInfo, ptr::null_mut};

use ntapi::ntioapi::IO_STATUS_BLOCK;
use toolkit::{ProcessEnvironmentBlock, ProcessSpawner, Sleeper, U16CStackString, println};
use winapi::{shared::{ntdef::{HANDLE, NTSTATUS, PVOID}, ntstatus::STATUS_INVALID_HANDLE}, um::{consoleapi::WriteConsoleW, processenv::GetStdHandle, winbase::STD_OUTPUT_HANDLE, wincon::SetConsoleOutputCP}};

use crate::child::ClientProcess;
extern crate builtins;

mod child;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {
    let peb = ProcessEnvironmentBlock::current_process();
    let args = peb.command_line();
    let result = unsafe { SetConsoleOutputCP(65001) };

    if let Some(arg) = args.at(1) {
       
        loop {
            // let peb = ProcessEnvironmentBlock::current_process();
            // let mut message = U16CStackString::<100>::from_str("test").unwrap();
            println!("test");

            Sleeper::sleep(1000);
        }

        return 0
    }

    let client = ClientProcess::create().unwrap();
    client.spawn_logger().unwrap();
    println!("test");

    Sleeper::sleep_infinite(1);

    0
}
