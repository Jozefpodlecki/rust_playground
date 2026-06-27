#![no_std]

use core::fmt::*;
use super::*;

pub trait Feature {
    fn bit(&self) -> u32;
}

impl Feature for StandardFeatureEdx {
    fn bit(&self) -> u32 { *self as u32 }
}

impl Feature for ExtendedFeatureEcx {
    fn bit(&self) -> u32 { *self as u32 }
}

impl Feature for ExtendedFeature7Eb {
    fn bit(&self) -> u32 { *self as u32 }
}

impl Feature for ExtendedFeature7Ec {
    fn bit(&self) -> u32 { *self as u32 }
}

impl Feature for ExtendedFeature7Ed {
    fn bit(&self) -> u32 { *self as u32 }
}

impl Feature for ExtendedFeature80000001Ecx {
    fn bit(&self) -> u32 { *self as u32 }
}

impl Feature for ExtendedFeature80000001Edx {
    fn bit(&self) -> u32 { *self as u32 }
}

pub trait FeatureSet {
    type Feature: Into<u32> + Copy;
    fn flags(&self) -> u32;
    fn features(&self) -> &[(Self::Feature, &str)];
}

pub struct StandardFeatures(pub u32);
pub struct ExtendedFeatures(pub u32);
pub struct Extended7EbFeatures(pub u32);
pub struct Extended7EcFeatures(pub u32);
pub struct Extended7EdFeatures(pub u32);
pub struct Extended80000001EcxFeatures(pub u32);
pub struct Extended80000001EdxFeatures(pub u32);

fn dump_feature_set<T: FeatureSet>(set: &T, f: &mut Formatter<'_>) -> Result {
    let flags = set.flags();
    let features = set.features();
    let mut first = true;
    let mut printed = false;
    
    for (feature, name) in features {
        let bit = (*feature).into();
        if flags & (1 << bit) != 0 {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}", name)?;
            first = false;
            printed = true;
        }
    }
    
    if !printed {
        write!(f, "(none)")?;
    }
    
    Ok(())
}

macro_rules! impl_feature_set {
    ($wrapper:ty, $enum:ty, $features:expr) => {
        impl FeatureSet for $wrapper {
            type Feature = $enum;
            fn flags(&self) -> u32 { self.0 }
            fn features(&self) -> &[(Self::Feature, &str)] {
                $features
            }
        }
        
        impl Display for $wrapper {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                dump_feature_set(self, f)
            }
        }
    };
}

impl_feature_set!(StandardFeatures, StandardFeatureEdx, {
    use StandardFeatureEdx::*;
    &[
        (Fpu, "FPU"), (Vme, "VME"), (De, "DE"), (Pse, "PSE"),
        (Tsc, "TSC"), (Msr, "MSR"), (Pae, "PAE"), (Mce, "MCE"),
        (Cx8, "CX8"), (Apic, "APIC"), (Sep, "SEP"), (Mtrr, "MTRR"),
        (Pge, "PGE"), (Mca, "MCA"), (Cmov, "CMOV"), (Pat, "PAT"),
        (Pse36, "PSE36"), (Psn, "PSN"), (Clfsh, "CLFSH"), (Ds, "DS"),
        (Acpi, "ACPI"), (Mmx, "MMX"), (Fxsr, "FXSR"), (Sse, "SSE"),
        (Sse2, "SSE2"), (Ss, "SS"), (Ht, "HT"), (Tm, "TM"),
        (Ia64, "IA64"), (Pbe, "PBE"),
    ]
});

impl_feature_set!(ExtendedFeatures, ExtendedFeatureEcx, {
    use ExtendedFeatureEcx::*;
    &[
        (Sse3, "SSE3"), (Pclmulqdq, "PCLMULQDQ"), (Dtes64, "DTES64"),
        (Monitor, "MONITOR"), (DsCpl, "DS-CPL"), (Vmx, "VMX"),
        (Smx, "SMX"), (Est, "EST"), (Tm2, "TM2"), (Sss, "SSSE3"),
        (CnxId, "CNXID"), (Fma, "FMA"), (Cx16, "CX16"), (Xtpr, "XTPR"),
        (Pdcm, "PDCM"), (Dca, "DCA"), (Sse4_1, "SSE4.1"), (Sse4_2, "SSE4.2"),
        (X2Apic, "X2APIC"), (Movbe, "MOVBE"), (Popcnt, "POPCNT"),
        (TscDeadline, "TSC-DEADLINE"), (Aes, "AES-NI"), (Xsave, "XSAVE"),
        (Osxsave, "OSXSAVE"), (Avx, "AVX"), (F16C, "F16C"),
        (Rdrnd, "RDRAND"), (Hypervisor, "HYPERVISOR"),
    ]
});

impl_feature_set!(Extended7EbFeatures, ExtendedFeature7Eb, {
    use ExtendedFeature7Eb::*;
    &[
        (Fsgsbase, "FSGSBASE"), (Ia32TsxAdjust, "TSX-ADJUST"), (Sgx, "SGX"),
        (Bmi1, "BMI1"), (Hle, "HLE"), (Avx2, "AVX2"), (FdpEx, "FDP-EX"),
        (Smep, "SMEP"), (Bmi2, "BMI2"), (Erms, "ERMS"), (Invpcid, "INVPCID"),
        (Rtm, "RTM"), (Pqm, "PQM"), (Mpx, "MPX"), (Pqe, "PQE"),
        (Avx512f, "AVX512F"), (Avx512dq, "AVX512DQ"), (Rdseed, "RDSEED"),
        (Adx, "ADX"), (Smap, "SMAP"), (Avx512ifma, "AVX512IFMA"),
        (Pcommit, "PCOMMIT"), (Clflushopt, "CLFLUSHOPT"), (Clwb, "CLWB"),
        (IntelPt, "INTEL-PT"), (Avx512pf, "AVX512PF"), (Avx512er, "AVX512ER"),
        (Avx512cd, "AVX512CD"), (Sha, "SHA"), (Avx512bw, "AVX512BW"),
        (Avx512vl, "AVX512VL"),
    ]
});

impl_feature_set!(Extended7EcFeatures, ExtendedFeature7Ec, {
    use ExtendedFeature7Ec::*;
    &[
        (Prefetchwt1, "PREFETCHWT1"), (Avx512vbmi, "AVX512VBMI"),
        (Umip, "UMIP"), (Pku, "PKU"), (Ospke, "OSPKU"), (Waitpkg, "WAITPKG"),
        (Avx512vbmi2, "AVX512VBMI2"), (CetSs, "CET-SS"), (Gfni, "GFNI"),
        (Vaes, "VAES"), (Vpclmulqdq, "VPCLMULQDQ"), (Avx512bitalg, "AVX512BITALG"),
        (Avx512vpopcntdq, "AVX512VPOPCNTDQ"), (Rdpid, "RDPID"),
    ]
});

impl_feature_set!(Extended7EdFeatures, ExtendedFeature7Ed, {
    use ExtendedFeature7Ed::*;
    &[(Lbrs, "LBRS"), (Cldemote, "CLDEMOTE"), (Mwaitu, "MWAITU")]
});

impl_feature_set!(Extended80000001EcxFeatures, ExtendedFeature80000001Ecx, {
    use ExtendedFeature80000001Ecx::*;
    &[
        (LahfSahf, "LAHF-SAHF"), (CmpLegacy, "CMP-LEGACY"), (Svmm, "SVMM"),
        (ExtApicSpace, "EXT-APIC-SPACE"), (Cr8Legacy, "CR8-LEGACY"), (Abm, "ABM"),
        (Sse4a, "SSE4A"), (MisalignSse, "MISALIGN-SSE"), (ThreeDNowPrefetch, "3DNOW-PREFETCH"),
        (OsVw, "OS-VW"), (Ibs, "IBS"), (Xop, "XOP"), (Skinit, "SKINIT"),
        (Wdt, "WDT"), (Lwp, "LWP"), (Fma4, "FMA4"), (Tce, "TCE"),
        (NodeId, "NODE-ID"), (Tbm, "TBM"), (TopologyExtensions, "TOPOLOGY-EXT"),
        (PerfCtrExtCore, "PERF-CTR-EXT-CORE"), (PerfCtrExtNb, "PERF-CTR-EXT-NB"),
        (BpExt, "BP-EXT"), (Mwaitx, "MWAITX"), (XSaveOpt, "XSAVE-OPT"),
    ]
});

impl_feature_set!(Extended80000001EdxFeatures, ExtendedFeature80000001Edx, {
    use ExtendedFeature80000001Edx::*;
    &[
        (Fpu, "FPU"), (Vme, "VME"), (De, "DE"), (Pse, "PSE"),
        (Tsc, "TSC"), (Msr, "MSR"), (Pae, "PAE"), (Mce, "MCE"),
        (Cx8, "CX8"), (Apic, "APIC"), (Syscall, "SYSCALL"), (Mtrr, "MTRR"),
        (Pge, "PGE"), (Mca, "MCA"), (Cmov, "CMOV"), (Pat, "PAT"),
        (Pse36, "PSE36"), (Mmu, "MMU"), (Fxsr, "FXSR"), (Mmx, "MMX"),
        (MmxExt, "MMX-EXT"), (Sse, "SSE"), (Sse2, "SSE2"), (Sse3, "SSE3"),
        (ThreeDNowExt, "3DNOW-EXT"),
    ]
});