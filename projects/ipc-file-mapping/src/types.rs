use core::fmt;

use ntapi::ntexapi::KUSER_SHARED_DATA;

use crate::utils::format_datetime_ms;

#[repr(transparent)]
pub struct KUserSharedData(KUSER_SHARED_DATA);

impl KUserSharedData {
    pub fn new() -> &'static Self {
        unsafe {
            &*(0x7FFE0000 as *const Self)
        }
    }

    pub fn system_time(&self) -> SystemTime {
        let time = &self.0.SystemTime;
        SystemTime(((time.High2Time as u64) << 32) | (time.LowPart as u64))
    }
}

#[repr(transparent)]
#[derive(Default, Clone, Copy)]
pub struct SystemTime(pub u64);

impl SystemTime {
    pub const WINDOWS_TO_UNIX_EPOCH: i64 = 116444736000000000;
    pub const WINDOWS_TO_UNIX_SECONDS: i64 = 11644473600;

    pub const fn new(time: u64) -> Self {
        Self(time)
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    pub const fn as_seconds(&self) -> u64 {
        self.0 / 10_000_000
    }

    pub const fn as_unix_seconds(&self) -> i64 {
        ((self.0 as i64 - Self::WINDOWS_TO_UNIX_EPOCH) / 10_000_000) as i64
    }

    pub const fn as_unix_nanos(&self) -> i128 {
        ((self.0 as i128 - Self::WINDOWS_TO_UNIX_EPOCH as i128) * 100) as i128
    }
}

pub enum Precision {
    Ms(u64),
    Us(u64),
    Ns(u64),
    Full(u64, u64, u64),
}

impl fmt::Display for SystemTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format_datetime_ms(self.0))
    }
}

impl fmt::Debug for SystemTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SystemTime({} - {})", self.0, format_datetime_ms(self.0))
    }
}

impl From<u64> for SystemTime {
    fn from(time: u64) -> Self {
        Self(time)
    }
}