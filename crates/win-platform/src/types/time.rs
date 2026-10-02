use core::fmt;

use crate::KUserSharedData;

const WINDOWS_TO_UNIX_SECONDS: i64 = 11_644_473_600;
const WINDOWS_TO_UNIX_TICKS: i64 = WINDOWS_TO_UNIX_SECONDS * 10_000_000;
const TICKS_PER_SECOND: i64 = 10_000_000;
const TICKS_PER_MILLISECOND: i64 = 10_000;
const SECONDS_PER_DAY: i64 = 86_400;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Iso8601([u8; Iso8601::LEN]);

impl Iso8601 {
    pub const LEN: usize = 24;

    pub const fn from_system_time(time: SystemTime) -> Self {
        let unix_nanos = time.as_unix_nanos();
        let total_seconds = unix_nanos.div_euclid(1_000_000_000) as i64;
        let millis = (unix_nanos.rem_euclid(1_000_000_000) / 1_000_000) as u32;

        let days = total_seconds.div_euclid(SECONDS_PER_DAY);
        let secs_of_day = total_seconds.rem_euclid(SECONDS_PER_DAY);

        let hours = (secs_of_day / 3_600) as u32;
        let minutes = ((secs_of_day % 3_600) / 60) as u32;
        let seconds = (secs_of_day % 60) as u32;

        let (year, month, day) = civil_from_days(days);

        let mut writer = Writer::new();

        writer.number(year as u32, 4);
        writer.char(b'-');
        writer.number(month, 2);
        writer.char(b'-');
        writer.number(day, 2);
        writer.char(b'T');
        writer.number(hours, 2);
        writer.char(b':');
        writer.number(minutes, 2);
        writer.char(b':');
        writer.number(seconds, 2);
        writer.char(b'.');
        writer.number(millis, 3);
        writer.char(b'Z');

        Self(writer.into_bytes())
    }

    pub const fn as_bytes(&self) -> &[u8; Self::LEN] {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.0) }
    }
}

struct Writer<const N: usize> {
    buf: [u8; N],
    cursor: usize,
}

impl<const N: usize> Writer<N> {
    const fn new() -> Self {
        Self { buf: [0u8; N], cursor: 0 }
    }

    const fn char(&mut self, ch: u8) {
        assert!(self.cursor < N);
        self.buf[self.cursor] = ch;
        self.cursor += 1;
    }

    const fn number(&mut self, mut value: u32, width: usize) {
        let end = self.cursor + width;
        let mut digit = end;
        while digit > self.cursor {
            digit -= 1;
            self.buf[digit] = b'0' + (value % 10) as u8;
            value /= 10;
        }
        self.cursor = end;
    }

    const fn into_bytes(self) -> [u8; N] {
        self.buf
    }
}

impl fmt::Display for Iso8601 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for Iso8601 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct SystemTime(pub u64);

impl SystemTime {
    pub const fn new(ticks: u64) -> Self {
        Self(ticks)
    }

    pub fn now() -> Self {
        KUserSharedData::system_time()
    }

    pub const fn as_ticks(&self) -> u64 {
        self.0
    }

    pub const fn as_filetime(&self) -> u64 {
        self.0
    }

    pub const fn as_windows_seconds(&self) -> u64 {
        self.0 / TICKS_PER_SECOND as u64
    }

    pub const fn as_unix_seconds(&self) -> i64 {
        (self.0 as i64 - WINDOWS_TO_UNIX_TICKS) / TICKS_PER_SECOND
    }

    pub const fn as_unix_nanos(&self) -> i128 {
        (self.0 as i128 - WINDOWS_TO_UNIX_TICKS as i128) * 100
    }

    pub const fn as_unix_millis(&self) -> i64 {
        (self.0 as i64 - WINDOWS_TO_UNIX_TICKS) / TICKS_PER_MILLISECOND
    }

    pub fn to_iso8601(&self) -> Iso8601 {
        Iso8601::from_system_time(*self)
    }

    pub fn to_filename_timestamp(&self) -> FilenameTimestamp {
        FilenameTimestamp::from_system_time(*self)
    }
}

impl fmt::Display for SystemTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.to_iso8601(), f)
    }
}

impl fmt::Debug for SystemTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.to_iso8601(), f)
    }
}

const fn write_padded(out: &mut [u8], i: usize, mut value: u32, width: usize) -> usize {
    let end = i + width;
    let mut j = end;
    while j > i {
        j -= 1;
        out[j] = b'0' + (value % 10) as u8;
        value /= 10;
    }

    end
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FilenameTimestamp([u8; FilenameTimestamp::LEN]);

impl FilenameTimestamp {
    pub const LEN: usize = 24;

    pub const fn from_system_time(time: SystemTime) -> Self {
        let unix_nanos = time.as_unix_nanos();
        let total_seconds = unix_nanos.div_euclid(1_000_000_000) as i64;
        let millis = (unix_nanos.rem_euclid(1_000_000_000) / 1_000_000) as u32;

        let days = total_seconds.div_euclid(SECONDS_PER_DAY);
        let secs_of_day = total_seconds.rem_euclid(SECONDS_PER_DAY);

        let hours = (secs_of_day / 3_600) as u32;
        let minutes = ((secs_of_day % 3_600) / 60) as u32;
        let seconds = (secs_of_day % 60) as u32;

        let (year, month, day) = civil_from_days(days);

        let mut writer = Writer::<{ Self::LEN }>::new();

        writer.number(year as u32, 4);
        writer.char(b'-');
        writer.number(month, 2);
        writer.char(b'-');
        writer.number(day, 2);
        writer.char(b'T');
        writer.number(hours, 2);
        writer.char(b'-');
        writer.number(minutes, 2);
        writer.char(b'-');
        writer.number(seconds, 2);
        writer.char(b'.');
        writer.number(millis, 3);
        writer.char(b'Z');

        Self(writer.into_bytes())
    }

    pub const fn as_bytes(&self) -> &[u8; Self::LEN] {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.0) }
    }
}

impl fmt::Display for FilenameTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for FilenameTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

const fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fmt(ticks: u64) -> &'static str {
        let s = SystemTime::new(ticks).to_iso8601();
        alloc::boxed::Box::leak(alloc::string::String::from(s.as_str()).into_boxed_str())
    }

    #[test]
    fn unix_epoch() {
        assert_eq!(fmt(WINDOWS_TO_UNIX_TICKS as u64), "1970-01-01T00:00:00.000Z");
    }

    #[test]
    fn known_timestamp() {
        let secs: i64 = 19_782 * 86_400 + 12 * 3_600 + 34 * 60 + 56;
        let ticks = (WINDOWS_TO_UNIX_SECONDS + secs) as u64 * 10_000_000 + 7_890_000;
        assert_eq!(fmt(ticks), "2024-02-29T12:34:56.789Z");
    }

    #[test]
    fn leap_year_2000() {
        let days: i64 = 11_016;
        let ticks = (WINDOWS_TO_UNIX_SECONDS + days * 86_400) as u64 * 10_000_000;
        assert_eq!(fmt(ticks), "2000-02-29T00:00:00.000Z");
    }

    #[test]
    fn non_leap_year_1900() {
        let days: i64 = -25_508;
        let ticks = (WINDOWS_TO_UNIX_SECONDS + days * 86_400) as u64 * 10_000_000;
        assert_eq!(fmt(ticks), "1900-03-01T00:00:00.000Z");
    }

    #[test]
    fn filename_timestamp_has_no_colons() {
        let ts = FilenameTimestamp::from_system_time(
            SystemTime::new((WINDOWS_TO_UNIX_SECONDS + 19_782 * 86_400) as u64 * 10_000_000),
        );
        assert!(!ts.as_str().contains(':'));
        assert_eq!(ts.as_str().len(), FilenameTimestamp::LEN);
    }
}