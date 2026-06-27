use core::arch::x86_64::CpuidResult;

use super::{ProcessorType, VendorID};

#[derive(Debug, Clone, Copy)]
pub struct CpuidBasicInfo {
    pub max_leaf: u32,
    pub vendor_id: VendorID,
    pub vendor_string: [u8; 12],
}

impl CpuidBasicInfo {
    pub fn from_leaf0(leaf: CpuidResult) -> Self {
        let ebx = leaf.ebx;
        let edx = leaf.edx;
        let ecx = leaf.ecx;
        
        let vendor_id = VendorID::from((ebx, edx, ecx));
        let vendor_string = Self::get_vendor_string(ebx, edx, ecx);
        
        CpuidBasicInfo {
            max_leaf: leaf.eax,
            vendor_id,
            vendor_string,
        }
    }
    
    fn get_vendor_string(ebx: u32, edx: u32, ecx: u32) -> [u8; 12] {
        let mut bytes = [0u8; 12];
        bytes[0..4].copy_from_slice(&ebx.to_le_bytes());
        bytes[4..8].copy_from_slice(&edx.to_le_bytes());
        bytes[8..12].copy_from_slice(&ecx.to_le_bytes());
        bytes
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CpuidProcessorInfo {
    pub family: u32,
    pub model: u32,
    pub stepping: u32,
    pub processor_type: ProcessorType,
    pub extended_family: u32,
    pub extended_model: u32,
    pub brand_index: u32,
}

impl From<CpuidResult> for CpuidProcessorInfo {
    fn from(leaf: CpuidResult) -> Self {
        let eax = leaf.eax;
        CpuidProcessorInfo {
            stepping: eax & 0xF,
            model: (eax >> 4) & 0xF,
            family: (eax >> 8) & 0xF,
            processor_type: match (eax >> 12) & 0x3 {
                0 => ProcessorType::OriginalOEM,
                1 => ProcessorType::Overdrive,
                2 => ProcessorType::DualCapable,
                _ => ProcessorType::Reserved,
            },
            extended_model: (eax >> 16) & 0xF,
            extended_family: (eax >> 20) & 0xFF,
            brand_index: (eax >> 24) & 0xFF,
        }
    }
}