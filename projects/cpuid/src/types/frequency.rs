use core::arch::x86_64::CpuidResult;

#[derive(Debug, Clone, Copy)]
pub struct CpuidFrequencyInfo {
    pub base_mhz: u32,
    pub max_mhz: u32,
    pub bus_mhz: u32,
    pub edx: u32,
}

impl CpuidFrequencyInfo {
    pub fn is_valid(&self) -> bool {
        self.base_mhz != 0 || self.max_mhz != 0 || self.bus_mhz != 0
    }
}

impl From<CpuidResult> for CpuidFrequencyInfo {
    fn from(leaf: CpuidResult) -> Self {
        CpuidFrequencyInfo {
            base_mhz: leaf.eax & 0xFFFF,
            max_mhz: leaf.ebx & 0xFFFF,
            bus_mhz: leaf.ecx & 0xFFFF,
            edx: leaf.edx,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CpuidTscInfo {
    pub denominator: u32,
    pub numerator: u32,
    pub nominal_frequency: u32,
}

impl From<CpuidResult> for CpuidTscInfo {
    fn from(leaf: CpuidResult) -> Self {
        CpuidTscInfo {
            denominator: leaf.eax,
            numerator: leaf.ebx,
            nominal_frequency: leaf.ecx,
        }
    }
}