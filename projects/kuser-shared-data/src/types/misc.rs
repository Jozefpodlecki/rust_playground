use core::fmt;

#[repr(transparent)]
pub struct PhysicalPages(pub u32);

impl PhysicalPages {
    pub const PAGE_SIZE: u64 = 4096;

    pub fn new(pages: u32) -> Self {
        Self(pages)
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }

    pub fn bytes(&self) -> u64 {
        self.0 as u64 * Self::PAGE_SIZE
    }

    pub fn kilobytes(&self) -> u64 {
        self.bytes() / 1024
    }

    pub fn megabytes(&self) -> u64 {
        self.bytes() / (1024 * 1024)
    }

    pub fn gigabytes(&self) -> u64 {
        self.bytes() / (1024 * 1024 * 1024)
    }

    pub fn format_memory(&self) -> impl fmt::Display + '_ {
        struct MemoryDisplay(u64);
        
        impl fmt::Display for MemoryDisplay {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let bytes = self.0;
                if bytes >= 1024 * 1024 * 1024 {
                    write!(f, "{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
                } else if bytes >= 1024 * 1024 {
                    write!(f, "{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
                } else if bytes >= 1024 {
                    write!(f, "{:.2} KB", bytes as f64 / 1024.0)
                } else {
                    write!(f, "{} B", bytes)
                }
            }
        }
        
        MemoryDisplay(self.bytes())
    }
}

impl fmt::Display for PhysicalPages {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} pages ({} bytes, {})", 
            self.0, 
            self.bytes(),
            self.format_memory()
        )
    }
}

impl fmt::Debug for PhysicalPages {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PhysicalPages({} pages = {})", self.0, self.format_memory())
    }
}

pub struct NtVersion {
    pub major: u32,
    pub minor: u32,
    pub build: u32,
}

impl fmt::Display for NtVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.build)
    }
}

