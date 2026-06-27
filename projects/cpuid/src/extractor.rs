use core::arch::x86_64::{__cpuid, __cpuid_count};
use crate::types::*;

type CpuidResult = core::arch::x86_64::CpuidResult;

pub struct CpuidExtractor;

impl CpuidExtractor {
    pub fn extract() -> FullCpuidInfo {
        let leaf0 = unsafe { __cpuid(0) };
        let max_leaf = leaf0.eax;
        
        let basic = CpuidBasicInfo::from_leaf0(leaf0);

        let leaf1 = unsafe { __cpuid(1) };
        let processor = leaf1.into();
        
        let leaf7_0 = unsafe { __cpuid_count(7, 0) };
        let leaf7_1 = if max_leaf >= 7 {
            unsafe { __cpuid_count(7, 1) }
        } else {
            CpuidResult { eax: 0, ebx: 0, ecx: 0, edx: 0 }
        };
        
        let features = CpuidFeatures::from_leaves(leaf1, leaf7_0, leaf7_1);

        let brand = CpuidBrandString::from_leaves([
            unsafe { __cpuid(0x80000002) },
            unsafe { __cpuid(0x80000003) },
            unsafe { __cpuid(0x80000004) },
        ]);

        let mut cache = [None; 10];
        for i in 0..10_usize {
            let cache_leaf = unsafe { __cpuid_count(4, i as u32) };
            if cache_leaf.eax & 0x1F != 0 {
                cache[i] = Some(cache_leaf.into());
            }
        }

        let topology = if max_leaf >= 0x1F {
            let top0 = unsafe { __cpuid_count(0x1F, 0) };
            let top1 = unsafe { __cpuid_count(0x1F, 1) };
            Some(CpuidTopologyInfo::from_leaves(top0, top1))
        } else if max_leaf >= 0xB {
            let top0 = unsafe { __cpuid_count(0xB, 0) };
            let top1 = unsafe { __cpuid_count(0xB, 1) };
            Some(CpuidTopologyInfo::from_leaves(top0, top1))
        } else {
            None
        };

        let address = if max_leaf >= 0x80000008 {
            Some(unsafe { __cpuid(0x80000008) }.into())
        } else {
            None
        };

        let power = if max_leaf >= 6 {
            Some(unsafe { __cpuid(6) }.into())
        } else {
            None
        };

        let frequency = if max_leaf >= 0x16 {
            let freq_leaf = unsafe { __cpuid(0x16) };
            if freq_leaf.eax != 0 || freq_leaf.ebx != 0 || freq_leaf.ecx != 0 {
                Some(freq_leaf.into())
            } else {
                None
            }
        } else {
            None
        };

         let tsc = if max_leaf >= 0x15 {
            let tsc_leaf = unsafe { __cpuid(0x15) };
            if tsc_leaf.eax != 0 || tsc_leaf.ebx != 0 || tsc_leaf.ecx != 0 {
                Some(tsc_leaf.into())
            } else {
                None
            }
        } else {
            None
        };

        let extended = CpuidExtendedInfo::from_max_leaf(
            unsafe { __cpuid(0x80000000) }.eax
        );

        FullCpuidInfo {
            basic,
            processor,
            features,
            brand,
            cache,
            topology,
            address,
            power,
            frequency,
            tsc,
            extended,
        }
    }
}