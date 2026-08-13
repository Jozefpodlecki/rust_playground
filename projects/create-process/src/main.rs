#![no_std]
#![no_main]
#![windows_subsystem = "console"]

use core::{mem, panic::PanicInfo, ptr};


use toolkit::*;

use crate::builder::ProcessBuilder;

mod types;
mod builder;

#[macro_use]
extern crate alloc;

extern crate builtins;

#[global_allocator]
static ALLOCATOR: FreeListAllocator1MB = FreeListAllocator1MB::new();


#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {

      let result = ProcessBuilder::<{1024 * 10}, {1024 * 3}, 512>::new()
        .with_absolute_image_path(r#"C:\Program Files (x86)\Steam\steamapps\common\Lost Ark\Binaries\Win64\LOSTARK.exe"#)
        .with_command_args("-PARAM21 -PARAM2=")
        // .with_parent_pid(parent_pid)
        .with_environment_from_peb()
        .with_process_group_from_peb()
        .suspended()
        .spawn();


    match result {
        Ok(info) => {
            println!("{info:?}");

        },
        Err(err) => {
            println!("{err}");
        },
    }

    0
}