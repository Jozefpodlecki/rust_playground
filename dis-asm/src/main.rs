#![feature(naked_functions_rustic_abi)]
#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![allow(unused_unsafe, unsafe_op_in_unsafe_fn)]

use core::{arch::{asm, naked_asm}, panic::PanicInfo};

mod crt;
mod rt_main;

#[cfg(not(test))]
#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
#[unsafe(naked)]
unsafe extern "C" fn mainCRTStartup() -> ! {
	naked_asm!(
		"sub rsp, 0x28",
        "call __security_init_cookie",
        "add rsp, 0x28",
        "jmp __scrt_common_main_seh",
	);
}