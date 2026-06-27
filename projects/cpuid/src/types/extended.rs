use core::arch::x86_64::{__cpuid, CpuidResult};

#[derive(Debug, Clone, Copy)]
pub struct CpuidExtendedFeatures {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
}

impl From<CpuidResult> for CpuidExtendedFeatures {
    fn from(leaf: CpuidResult) -> Self {
        CpuidExtendedFeatures {
            eax: leaf.eax,
            ebx: leaf.ebx,
            ecx: leaf.ecx,
            edx: leaf.edx,
        }
    }
}

// L1 Cache/TLB Info (0x80000005)
#[derive(Debug, Clone, Copy)]
pub struct CpuidL1CacheTlbInfo {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
    // Decoded fields
    pub l1_data_cache_size: u32,      // KB
    pub l1_data_cache_associativity: u32,
    pub l1_data_cache_lines_per_tag: u32,
    pub l1_data_cache_line_size: u32,
    pub l1_instruction_cache_size: u32, // KB
    pub l1_instruction_cache_associativity: u32,
    pub l1_instruction_cache_lines_per_tag: u32,
    pub l1_instruction_cache_line_size: u32,
}

impl From<CpuidResult> for CpuidL1CacheTlbInfo {
    fn from(leaf: CpuidResult) -> Self {
        // ECX: L1 Data Cache
        let data_cache_size = ((leaf.ecx >> 24) & 0xFF) * 1024 / 1024; // KB
        let data_cache_associativity = (leaf.ecx >> 16) & 0xFF;
        let data_cache_lines_per_tag = (leaf.ecx >> 8) & 0xFF;
        let data_cache_line_size = leaf.ecx & 0xFF;
        
        // EDX: L1 Instruction Cache
        let inst_cache_size = ((leaf.edx >> 24) & 0xFF) * 1024 / 1024; // KB
        let inst_cache_associativity = (leaf.edx >> 16) & 0xFF;
        let inst_cache_lines_per_tag = (leaf.edx >> 8) & 0xFF;
        let inst_cache_line_size = leaf.edx & 0xFF;
        
        CpuidL1CacheTlbInfo {
            eax: leaf.eax,
            ebx: leaf.ebx,
            ecx: leaf.ecx,
            edx: leaf.edx,
            l1_data_cache_size: data_cache_size,
            l1_data_cache_associativity: data_cache_associativity,
            l1_data_cache_lines_per_tag: data_cache_lines_per_tag,
            l1_data_cache_line_size: data_cache_line_size,
            l1_instruction_cache_size: inst_cache_size,
            l1_instruction_cache_associativity: inst_cache_associativity,
            l1_instruction_cache_lines_per_tag: inst_cache_lines_per_tag,
            l1_instruction_cache_line_size: inst_cache_line_size,
        }
    }
}

// L2 Cache/TLB Info (0x80000006)
#[derive(Debug, Clone, Copy)]
pub struct CpuidL2CacheTlbInfo {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
    // Decoded fields
    pub l2_cache_size: u32,           // KB
    pub l2_cache_associativity: u32,
    pub l2_cache_line_size: u32,
    // L2 TLB
    pub l2_tlb_2m_4m_associativity: u32,
    pub l2_tlb_2m_4m_entries: u32,
    pub l2_tlb_4k_associativity: u32,
    pub l2_tlb_4k_entries: u32,
}

impl From<CpuidResult> for CpuidL2CacheTlbInfo {
    fn from(leaf: CpuidResult) -> Self {
        // ECX: L2 Cache
        let cache_size = ((leaf.ecx >> 16) & 0xFFFF) * 1024 / 1024; // KB
        let cache_associativity = (leaf.ecx >> 12) & 0xF;
        let cache_line_size = leaf.ecx & 0xFF;
        
        // EAX: L2 TLB 2M/4M
        let tlb_2m_4m_associativity = (leaf.eax >> 16) & 0xFF;
        let tlb_2m_4m_entries = leaf.eax & 0xFFFF;
        
        // EBX: L2 TLB 4K
        let tlb_4k_associativity = (leaf.ebx >> 16) & 0xFF;
        let tlb_4k_entries = leaf.ebx & 0xFFFF;
        
        CpuidL2CacheTlbInfo {
            eax: leaf.eax,
            ebx: leaf.ebx,
            ecx: leaf.ecx,
            edx: leaf.edx,
            l2_cache_size: cache_size,
            l2_cache_associativity: cache_associativity,
            l2_cache_line_size: cache_line_size,
            l2_tlb_2m_4m_associativity: tlb_2m_4m_associativity,
            l2_tlb_2m_4m_entries: tlb_2m_4m_entries,
            l2_tlb_4k_associativity: tlb_4k_associativity,
            l2_tlb_4k_entries: tlb_4k_entries,
        }
    }
}

// Extended Power Management (0x80000007)
#[derive(Debug, Clone, Copy)]
pub struct CpuidExtendedPowerMgmt {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
    // Decoded EDX bits
    pub invariant_tsc: bool,
    pub thermal_monitor: bool,
    pub software_thermal_control: bool,
    pub hardware_thermal_control: bool,
    pub performance_status: bool,
    pub hardware_p_state: bool,
    pub software_p_state: bool,
    pub tsc_scale: bool,
}

impl From<CpuidResult> for CpuidExtendedPowerMgmt {
    fn from(leaf: CpuidResult) -> Self {
        let edx = leaf.edx;
        CpuidExtendedPowerMgmt {
            eax: leaf.eax,
            ebx: leaf.ebx,
            ecx: leaf.ecx,
            edx: leaf.edx,
            invariant_tsc: (edx & (1 << 8)) != 0,
            thermal_monitor: (edx & (1 << 0)) != 0,
            software_thermal_control: (edx & (1 << 1)) != 0,
            hardware_thermal_control: (edx & (1 << 2)) != 0,
            performance_status: (edx & (1 << 3)) != 0,
            hardware_p_state: (edx & (1 << 4)) != 0,
            software_p_state: (edx & (1 << 5)) != 0,
            tsc_scale: (edx & (1 << 6)) != 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CpuidMemEncryptionInfo {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
}

impl From<CpuidResult> for CpuidMemEncryptionInfo {
    fn from(leaf: CpuidResult) -> Self {
        CpuidMemEncryptionInfo {
            eax: leaf.eax,
            ebx: leaf.ebx,
            ecx: leaf.ecx,
            edx: leaf.edx,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CpuidExtendedInfo {
    pub max_extended_leaf: u32,
    pub extended_features: Option<CpuidExtendedFeatures>,
    pub l1_cache_tlb: Option<CpuidL1CacheTlbInfo>,
    pub l2_cache_tlb: Option<CpuidL2CacheTlbInfo>,
    pub power_management: Option<CpuidExtendedPowerMgmt>,
    pub memory_encryption: Option<CpuidMemEncryptionInfo>,
}

impl CpuidExtendedInfo {
    pub fn from_max_leaf(max_ext: u32) -> Self {
        let mut info = CpuidExtendedInfo {
            max_extended_leaf: max_ext,
            extended_features: None,
            l1_cache_tlb: None,
            l2_cache_tlb: None,
            power_management: None,
            memory_encryption: None,
        };

        if max_ext >= 0x80000001 {
            info.extended_features = Some(unsafe { __cpuid(0x80000001) }.into());
        }

        if max_ext >= 0x80000005 {
            info.l1_cache_tlb = Some(unsafe { __cpuid(0x80000005) }.into());
        }

        if max_ext >= 0x80000006 {
            info.l2_cache_tlb = Some(unsafe { __cpuid(0x80000006) }.into());
        }

        if max_ext >= 0x80000007 {
            info.power_management = Some(unsafe { __cpuid(0x80000007) }.into());
        }

        if max_ext >= 0x8000001F {
            info.memory_encryption = Some(unsafe { __cpuid(0x8000001F) }.into());
        }

        info
    }
}
