use core::panic::PanicInfo;

use crate::{STATUS_FATAL_APP_EXIT, syscalls::NtTerminateProcess, types::NtCurrentProcess};

pub fn default_panic_handler(info: &PanicInfo) -> ! {

    NtTerminateProcess(NtCurrentProcess, STATUS_FATAL_APP_EXIT)
}