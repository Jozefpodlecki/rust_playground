#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![allow(unused)]

use core::panic::PanicInfo;

use ntapi::ntexapi::KUSER_SHARED_DATA;

mod kuser;
mod types;

use utils::*;

use crate::kuser::KUserSharedData;

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {

    let kuser = KUserSharedData::new();

    println!("{}", kuser);

    loop {
        println!("{}", kuser.system_time());
        Sleeper::sleep(1000);
    }

    0
}