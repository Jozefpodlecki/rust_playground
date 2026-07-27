#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![feature(core_intrinsics)]

use core::{intrinsics::unreachable, panic::PanicInfo, ptr::null_mut};

use ntapi::ntpsapi::NtCurrentProcess;
use toolkit::{File, ProcessEnvironmentBlock, println, syscalls::NtTerminateProcess};

use crate::utils::*;


mod utils;

extern crate builtins;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    unsafe {
        NtTerminateProcess(NtCurrentProcess, 0);
        unreachable();
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {

    enable_manage_volume_privilege();

    let peb = ProcessEnvironmentBlock::current_process();

    let mut data_path = peb.executable_path().parent().to_owned::<200>();
    data_path.prepend(r#"\??\"#);
    data_path.join(r#"test.data"#);
    let file = File::create(data_path).unwrap();

    let handle = file.handle();
    let size: u64 = 1024 * 1024 * 1024;

    if !create_sparse_file_ntapi(handle, size) {
        return 1;
    }

    0
}