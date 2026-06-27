#![no_std]

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VendorID {
    Intel,
    AMD,
    Cyrix,
    Centaur,
    Unknown,
}

impl VendorID {
    pub fn as_str(&self) -> &'static str {
        match self {
            VendorID::Intel => "Intel",
            VendorID::AMD => "AMD",
            VendorID::Cyrix => "Cyrix",
            VendorID::Centaur => "Centaur",
            VendorID::Unknown => "Unknown",
        }
    }
}

impl From<(u32, u32, u32)> for VendorID {
    fn from(regs: (u32, u32, u32)) -> Self {
        let (ebx, edx, ecx) = regs;
        let mut bytes = [0u8; 12];
        bytes[0..4].copy_from_slice(&ebx.to_le_bytes());
        bytes[4..8].copy_from_slice(&edx.to_le_bytes());
        bytes[8..12].copy_from_slice(&ecx.to_le_bytes());
        
        match &bytes {
            b"GenuineIntel" => VendorID::Intel,
            b"AuthenticAMD" => VendorID::AMD,
            b"CyrixInstead" => VendorID::Cyrix,
            b"CentaurHauls" => VendorID::Centaur,
            _ => VendorID::Unknown,
        }
    }
}

impl From<[u32; 3]> for VendorID {
    fn from(regs: [u32; 3]) -> Self {
        VendorID::from((regs[0], regs[1], regs[2]))
    }
}