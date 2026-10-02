use core::time::Duration;

use winapi::um::winnt::LARGE_INTEGER;

use crate::{extensions::DurationExtensions, syscalls::NtDelayExecution};

pub struct Sleeper;

impl Sleeper {
    pub fn sleep(duration: Duration) {
        let mut li = duration.to_large_integer();
        NtDelayExecution(1, &mut li);
    }

    pub fn sleep_infinite(alertable: u8) {
        let mut li = Duration::MAX.to_large_integer();
        NtDelayExecution(alertable, &mut li);
    }
}