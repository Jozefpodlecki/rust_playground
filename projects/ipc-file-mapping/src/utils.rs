use core::fmt;

use crate::types::{Precision, SystemTime};

pub fn unix_seconds_to_datetime(unix_seconds: i64, f: &mut fmt::Formatter<'_>, precision: Precision) -> fmt::Result {
    let days = unix_seconds / 86400;
    let seconds_in_day = unix_seconds % 86400;
    let hours = seconds_in_day / 3600;
    let minutes = (seconds_in_day % 3600) / 60;
    let secs = seconds_in_day % 60;

    let mut year = 1970;
    let mut remaining_days = days;

    while remaining_days >= 365 {
        let leap = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) { 366 } else { 365 };
        if remaining_days < leap {
            break;
        }
        remaining_days -= leap;
        year += 1;
    }

    let month_days = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    let mut day = remaining_days;
    for (i, &days_in_month) in month_days.iter().enumerate() {
        if day < days_in_month {
            month = i + 1;
            break;
        }
        day -= days_in_month;
    }
    let day = day + 1;

    match precision {
        Precision::Ms(ms) => write!(f, "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
            year, month, day, hours, minutes, secs, ms),
        Precision::Us(us) => write!(f, "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:06}",
            year, month, day, hours, minutes, secs, us),
        Precision::Ns(ns) => write!(f, "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:09}",
            year, month, day, hours, minutes, secs, ns),
        Precision::Full(ms, us, ns) => write!(f, "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}:{:03}:{:03}",
            year, month, day, hours, minutes, secs, ms, us, ns),
    }
}

pub fn split_time(value: u64) -> (u64, u64, u64, u64, u64) {
    let seconds = value / 10_000_000;
    let fractional_100ns = value % 10_000_000;
    let milliseconds = fractional_100ns / 10_000;
    let microseconds = (fractional_100ns % 10_000) / 10;
    let nanoseconds = (fractional_100ns % 10) * 100;
    (seconds, milliseconds, microseconds, nanoseconds, fractional_100ns)
}

pub fn format_datetime(value: u64) -> impl fmt::Display {
    struct DateTimeDisplay(u64);
    
    impl fmt::Display for DateTimeDisplay {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let seconds = self.0 / 10_000_000;
            let fractional_100ns = self.0 % 10_000_000;
            let milliseconds = fractional_100ns / 10_000;
            let microseconds = (fractional_100ns % 10_000) / 10;
            let nanoseconds = (fractional_100ns % 10) * 100;
            
            let unix_seconds = seconds as i64 - SystemTime::WINDOWS_TO_UNIX_SECONDS;
            unix_seconds_to_datetime(unix_seconds, f, Precision::Full(milliseconds, microseconds, nanoseconds))
        }
    }
    
    DateTimeDisplay(value)
}

pub fn format_datetime_ms(value: u64) -> impl fmt::Display {
    struct DateTimeMsDisplay(u64);
    
    impl fmt::Display for DateTimeMsDisplay {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let seconds = self.0 / 10_000_000;
            let fractional_100ns = self.0 % 10_000_000;
            let milliseconds = fractional_100ns / 10_000;
            
            let unix_seconds = seconds as i64 - SystemTime::WINDOWS_TO_UNIX_SECONDS;
            unix_seconds_to_datetime(unix_seconds, f, Precision::Ms(milliseconds))
        }
    }
    
    DateTimeMsDisplay(value)
}

pub fn format_datetime_us(value: u64) -> impl fmt::Display {
    struct DateTimeUsDisplay(u64);
    
    impl fmt::Display for DateTimeUsDisplay {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let seconds = self.0 / 10_000_000;
            let fractional_100ns = self.0 % 10_000_000;
            let microseconds = fractional_100ns / 10;
            
            let unix_seconds = seconds as i64 - SystemTime::WINDOWS_TO_UNIX_SECONDS;
            unix_seconds_to_datetime(unix_seconds, f, Precision::Us(microseconds))
        }
    }
    
    DateTimeUsDisplay(value)
}

pub fn format_datetime_ns(value: u64) -> impl fmt::Display {
    struct DateTimeNsDisplay(u64);
    
    impl fmt::Display for DateTimeNsDisplay {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let seconds = self.0 / 10_000_000;
            let fractional_100ns = self.0 % 10_000_000;
            let nanoseconds = fractional_100ns * 100;
            
            let unix_seconds = seconds as i64 - SystemTime::WINDOWS_TO_UNIX_SECONDS;
            unix_seconds_to_datetime(unix_seconds, f, Precision::Ns(nanoseconds))
        }
    }
    
    DateTimeNsDisplay(value)
}