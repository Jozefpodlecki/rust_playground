use core::time::Duration;

use winapi::um::winnt::LARGE_INTEGER;

use crate::syscalls::NtDelayExecution;

pub struct Sleeper;

impl Sleeper {
    pub fn sleep(duration: Duration) {
        let mut delay: LARGE_INTEGER = unsafe { core::mem::zeroed() };
        unsafe {
            let milliseconds = duration.as_millis();
            *delay.QuadPart_mut() = -(milliseconds as i64) * 10_000;
            NtDelayExecution(0, &mut delay);
        }
    }

    pub fn sleep_infinite(alertable: u8) {
        let mut delay: LARGE_INTEGER = unsafe { core::mem::zeroed() };
        unsafe {
            *delay.QuadPart_mut() = i64::MIN;
            NtDelayExecution(alertable, &mut delay);
        }
    }
}