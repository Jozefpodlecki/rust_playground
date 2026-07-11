use core::fmt;
use core::fmt::Write;

use ntapi::ntexapi::KUSER_SHARED_DATA;

pub struct SystemTime(ntapi::ntapi_base::KSYSTEM_TIME);

impl SystemTime {
    pub const fn now() -> Self {
        const KUSER: *const KUSER_SHARED_DATA = 0x7FFE0000 as *const KUSER_SHARED_DATA;
        unsafe { Self((*KUSER).SystemTime) }
    }

    pub const fn as_u64(&self) -> u64 {
        let low = self.0.LowPart as u64;
        let high = self.0.High1Time as u64;
        (high << 32) | low
    }

    pub const fn to_seconds(&self) -> u64 {
        self.as_u64() / 10_000_000
    }

    pub const fn to_millis(&self) -> u64 {
        self.as_u64() / 10_000
    }

    pub const fn to_unix_seconds(&self) -> u64 {
        const WINDOWS_EPOCH_TO_UNIX: u64 = 11_644_473_600;
        self.as_u64() / 10_000_000 - WINDOWS_EPOCH_TO_UNIX
    }

    pub const fn to_date_components(&self) -> (u16, u8, u8, u8, u8, u8, u32) {
        let secs = self.to_unix_seconds();
        let days = secs / 86400;
        let secs_in_day = secs % 86400;
        
        let hour = (secs_in_day / 3600) as u8;
        let minute = ((secs_in_day % 3600) / 60) as u8;
        let second = (secs_in_day % 60) as u8;
        let millis = (self.as_u64() % 10_000_000) / 10_000;
        
        let (year, month, day) = Self::days_to_date(days);
        (year, month, day, hour, minute, second, millis as u32)
    }

    pub fn to_hh_mm_ss(&self) -> heapless::String<8> {
        let (_, _, _, hour, minute, second, _) = self.to_date_components();
        let mut s = heapless::String::new();
        let _ = write!(&mut s, "{:02}{:02}{:02}", hour, minute, second);
        s
    }

    pub const fn days_to_date(mut days: u64) -> (u16, u8, u8) {
        let mut year = 1970;
        let mut days_in_year;
        
        loop {
            days_in_year = if Self::is_leap_year(year) { 366 } else { 365 };
            if days < days_in_year {
                break;
            }
            days -= days_in_year;
            year += 1;
        }
        
        const MONTH_DAYS: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        
        let mut month = 1;
        let mut idx = 0;
        while idx < 12 {
            let days_in_month = if month == 2 && Self::is_leap_year(year) {
                29
            } else {
                MONTH_DAYS[idx] as u64
            };
            
            if days < days_in_month {
                break;
            }
            days -= days_in_month;
            month += 1;
            idx += 1;
        }
        
        (year as u16, month as u8, (days + 1) as u8)
    }

    pub const fn is_leap_year(year: u64) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }
}

impl fmt::Display for SystemTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (year, month, day, hour, minute, second, millis) = self.to_date_components();
        write!(
            f,
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
            year, month, day, hour, minute, second, millis
        )
    }
}