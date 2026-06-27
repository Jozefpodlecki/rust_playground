use core::arch::x86_64::CpuidResult;

use crate::types::*;


#[derive(Debug, Clone, Copy)]
pub struct CpuidFeatures {
    pub standard_edx: u32,
    pub extended_ecx: u32,
    pub extended_7_ebx: u32,
    pub extended_7_ecx: u32,
    pub extended_7_edx: u32,
    pub extended_7_1_eax: u32,
    pub extended_7_1_ebx: u32,
    pub extended_7_1_ecx: u32,
    pub extended_7_1_edx: u32,
}

impl CpuidFeatures {
    pub fn from_leaves(leaf1: CpuidResult, leaf7_0: CpuidResult, leaf7_1: CpuidResult) -> Self {
        CpuidFeatures {
            standard_edx: leaf1.edx,
            extended_ecx: leaf1.ecx,
            extended_7_ebx: leaf7_0.ebx,
            extended_7_ecx: leaf7_0.ecx,
            extended_7_edx: leaf7_0.edx,
            extended_7_1_eax: leaf7_1.eax,
            extended_7_1_ebx: leaf7_1.ebx,
            extended_7_1_ecx: leaf7_1.ecx,
            extended_7_1_edx: leaf7_1.edx,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CpuidCacheInfo {
    pub cache_type: u8,
    pub cache_level: u8,
    pub cache_size_kb: u32,
    pub ways_of_associativity: u32,
    pub line_size_bytes: u32,
    pub sets: u32,
    pub partitions: u32,
}

impl From<CpuidResult> for CpuidCacheInfo {
    fn from(leaf: CpuidResult) -> Self {
        let cache_type = leaf.eax & 0x1F;
        let cache_level = (leaf.eax >> 5) & 0x7;
        let line_size = (leaf.ebx & 0xFFF) + 1;
        let partitions = ((leaf.ebx >> 12) & 0x3FF) + 1;
        let ways = ((leaf.ebx >> 22) & 0x3FF) + 1;
        let sets = leaf.ecx + 1;
        let cache_size_kb = (ways * partitions * line_size * sets) / 1024;

        CpuidCacheInfo {
            cache_type: cache_type as u8,
            cache_level: cache_level as u8,
            cache_size_kb,
            ways_of_associativity: ways,
            line_size_bytes: line_size,
            sets,
            partitions,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CpuidTopologyInfo {
    pub smt_mask: u32,
    pub core_mask: u32,
    pub smt_shift: u32,
    pub core_shift: u32,
}

impl CpuidTopologyInfo {
    pub fn from_leaves(leaf0: CpuidResult, leaf1: CpuidResult) -> Self {
        CpuidTopologyInfo {
            smt_mask: (1 << (leaf0.eax & 0x1F)) - 1,
            core_mask: (1 << (leaf1.eax & 0x1F)) - 1,
            smt_shift: leaf0.eax & 0x1F,
            core_shift: leaf1.eax & 0x1F,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CpuidAddressInfo {
    pub physical_addr_bits: u32,
    pub virtual_addr_bits: u32,
}

impl From<CpuidResult> for CpuidAddressInfo {
    fn from(leaf: CpuidResult) -> Self {
        CpuidAddressInfo {
            physical_addr_bits: leaf.eax & 0xFF,
            virtual_addr_bits: (leaf.eax >> 8) & 0xFF,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CpuidBrandString {
    pub brand: [u8; 48],
}

impl CpuidBrandString {
    pub fn as_str(&self) -> Result<&str, core::str::Utf8Error> {
        let len = self.brand.iter()
            .position(|&b| b == 0)
            .unwrap_or(self.brand.len());
        core::str::from_utf8(&self.brand[..len])
    }
    
    pub fn from_leaves(leaves: [CpuidResult; 3]) -> Self {
        let mut brand = [0u8; 48];
        for (i, leaf) in leaves.iter().enumerate() {
            let offset = i * 16;
            brand[offset..offset+4].copy_from_slice(&leaf.eax.to_le_bytes());
            brand[offset+4..offset+8].copy_from_slice(&leaf.ebx.to_le_bytes());
            brand[offset+8..offset+12].copy_from_slice(&leaf.ecx.to_le_bytes());
            brand[offset+12..offset+16].copy_from_slice(&leaf.edx.to_le_bytes());
        }
        CpuidBrandString { brand }
    }
}

#[derive(Debug, Clone)]
pub struct FullCpuidInfo {
    pub basic: CpuidBasicInfo,
    pub processor: CpuidProcessorInfo,
    pub features: CpuidFeatures,
    pub brand: CpuidBrandString,
    pub cache: [Option<CpuidCacheInfo>; 10],
    pub topology: Option<CpuidTopologyInfo>,
    pub address: Option<CpuidAddressInfo>,
    pub power: Option<CpuidPowerInfo>,
    pub frequency: Option<CpuidFrequencyInfo>,
    pub tsc: Option<CpuidTscInfo>,
    pub extended: CpuidExtendedInfo,
}

impl FullCpuidInfo {
    pub fn has_avx2(&self) -> bool {
        (self.features.extended_7_ebx & (1 << 5)) != 0
    }
    
    pub fn has_avx512f(&self) -> bool {
        (self.features.extended_7_ebx & (1 << 16)) != 0
    }
    
    pub fn has_sse4_2(&self) -> bool {
        (self.features.extended_ecx & (1 << 20)) != 0
    }
    
    pub fn has_aes_ni(&self) -> bool {
        (self.features.extended_ecx & (1 << 25)) != 0
    }
    
    pub fn has_rdrand(&self) -> bool {
        (self.features.extended_ecx & (1 << 30)) != 0
    }
    
    pub fn is_virtualized(&self) -> bool {
        (self.features.extended_ecx & (1 << 31)) != 0
    }
    
    pub fn has_hyperthreading(&self) -> bool {
        (self.features.standard_edx & (1 << 28)) != 0
    }
    
    pub fn supports_64bit(&self) -> bool {
        if let Some(addr) = &self.address {
            addr.virtual_addr_bits >= 48
        } else {
            false
        }
    }
    
    pub fn get_total_logical_cores(&self) -> Option<u32> {
        self.topology.map(|top| {
            let smt = top.smt_mask + 1;
            let cores = top.core_mask + 1;
            smt * cores
        })
    }
    
    pub fn get_l1_cache(&self) -> Option<&CpuidCacheInfo> {
        self.cache.iter().find(|c| {
            c.as_ref().map_or(false, |info| info.cache_level == 1)
        }).and_then(|c| c.as_ref())
    }
    
    pub fn get_l2_cache(&self) -> Option<&CpuidCacheInfo> {
        self.cache.iter().find(|c| {
            c.as_ref().map_or(false, |info| info.cache_level == 2)
        }).and_then(|c| c.as_ref())
    }
    
    pub fn get_l3_cache(&self) -> Option<&CpuidCacheInfo> {
        self.cache.iter().find(|c| {
            c.as_ref().map_or(false, |info| info.cache_level == 3)
        }).and_then(|c| c.as_ref())
    }
}