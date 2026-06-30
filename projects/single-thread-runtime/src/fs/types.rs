use core::fmt;

#[repr(C)]
pub struct FileMetadata {
    pub size: FileSize,
    pub creation_time: FileTime,
    pub last_access_time: FileTime,
    pub last_write_time: FileTime,
    pub attributes: FileAttributes,
}

#[derive(Clone, Copy)]
pub struct FileSize(pub u64);

#[derive(Clone, Copy)]
pub struct FileTime(pub u64);

#[derive(Clone, Copy)]
pub struct FileAttributes(pub u32);

impl FileMetadata {
    pub fn new() -> Self {
        Self {
            size: FileSize(0),
            creation_time: FileTime(0),
            last_access_time: FileTime(0),
            last_write_time: FileTime(0),
            attributes: FileAttributes(0),
        }
    }
}

impl fmt::Display for FileSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let size = self.0;
        if size < 1024 {
            write!(f, "{} B", size)
        } else if size < 1024 * 1024 {
            write!(f, "{:.2} KB", size as f64 / 1024.0)
        } else if size < 1024 * 1024 * 1024 {
            write!(f, "{:.2} MB", size as f64 / (1024.0 * 1024.0))
        } else {
            write!(f, "{:.2} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

fn is_leap_year(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

impl fmt::Display for FileTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let time = self.0;
        let seconds = time / 10_000_000;
        const EPOCH_DIFF: u64 = 11644473600;
        
        let unix_time = if seconds >= EPOCH_DIFF {
            seconds - EPOCH_DIFF
        } else {
            0
        };
        
        let days = unix_time / 86400;
        let seconds_in_day = unix_time % 86400;
        
        let hours = seconds_in_day / 3600;
        let minutes = (seconds_in_day % 3600) / 60;
        let secs = seconds_in_day % 60;
        
        let mut year = 1970;
        let mut remaining_days = days;
        let month_days = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        
        loop {
            let days_in_year = if is_leap_year(year) { 366 } else { 365 };
            if remaining_days < days_in_year {
                break;
            }
            remaining_days -= days_in_year;
            year += 1;
        }
        
        let mut month = 1;
        for &days_in_month in month_days.iter() {
            let days_in_month = if month == 2 && is_leap_year(year) { 29 } else { days_in_month };
            if remaining_days < days_in_month {
                break;
            }
            remaining_days -= days_in_month;
            month += 1;
        }
        
        let day = remaining_days + 1;
        
        write!(
            f,
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            year, month, day, hours, minutes, secs
        )
    }
}

impl fmt::Display for FileAttributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        let attrs = self.0;
        
        if attrs & 0x1 != 0 {
            write!(f, "READONLY")?;
            first = false;
        }
        if attrs & 0x2 != 0 {
            if !first { write!(f, " | ")?; }
            write!(f, "HIDDEN")?;
            first = false;
        }
        if attrs & 0x4 != 0 {
            if !first { write!(f, " | ")?; }
            write!(f, "SYSTEM")?;
            first = false;
        }
        if attrs & 0x10 != 0 {
            if !first { write!(f, " | ")?; }
            write!(f, "DIRECTORY")?;
            first = false;
        }
        if attrs & 0x20 != 0 {
            if !first { write!(f, " | ")?; }
            write!(f, "ARCHIVE")?;
            first = false;
        }
        if attrs & 0x80 != 0 {
            if !first { write!(f, " | ")?; }
            write!(f, "NORMAL")?;
            first = false;
        }
        if attrs & 0x100 != 0 {
            if !first { write!(f, " | ")?; }
            write!(f, "TEMPORARY")?;
            first = false;
        }
        if attrs & 0x800 != 0 {
            if !first { write!(f, " | ")?; }
            write!(f, "COMPRESSED")?;
            first = false;
        }
        if attrs & 0x1000 != 0 {
            if !first { write!(f, " | ")?; }
            write!(f, "OFFLINE")?;
            first = false;
        }
        if attrs & 0x2000 != 0 {
            if !first { write!(f, " | ")?; }
            write!(f, "NOT_CONTENT_INDEXED")?;
            first = false;
        }
        if attrs & 0x4000 != 0 {
            if !first { write!(f, " | ")?; }
            write!(f, "ENCRYPTED")?;
            first = false;
        }
        
        if first {
            write!(f, "0x{:08X}", attrs)?;
        }
        
        Ok(())
    }
}

impl fmt::Display for FileMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "File Metadata:")?;
        writeln!(f, "  Size: {}", self.size)?;
        writeln!(f, "  Creation Time: {}", self.creation_time)?;
        writeln!(f, "  Last Access: {}", self.last_access_time)?;
        writeln!(f, "  Last Write: {}", self.last_write_time)?;
        writeln!(f, "  Attributes: {}", self.attributes)?;
        Ok(())
    }
}