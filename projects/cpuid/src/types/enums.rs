use core::fmt::*;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessorType {
    OriginalOEM = 0,
    Overdrive = 1,
    DualCapable = 2,
    Reserved = 3,
}

impl ProcessorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProcessorType::OriginalOEM => "Original OEM",
            ProcessorType::Overdrive => "Overdrive",
            ProcessorType::DualCapable => "Dual Capable",
            ProcessorType::Reserved => "Reserved",
        }
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandardFeatureEdx {
    Fpu = 0, Vme = 1, De = 2, Pse = 3, Tsc = 4, Msr = 5, Pae = 6, Mce = 7,
    Cx8 = 8, Apic = 9, Sep = 11, Mtrr = 12, Pge = 13, Mca = 14, Cmov = 15,
    Pat = 16, Pse36 = 17, Psn = 18, Clfsh = 19, Ds = 21, Acpi = 22, Mmx = 23,
    Fxsr = 24, Sse = 25, Sse2 = 26, Ss = 27, Ht = 28, Tm = 29, Ia64 = 30, Pbe = 31,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtendedFeatureEcx {
    Sse3 = 0, Pclmulqdq = 1, Dtes64 = 2, Monitor = 3, DsCpl = 4, Vmx = 5, Smx = 6,
    Est = 7, Tm2 = 8, Sss = 9, CnxId = 10, Fma = 12, Cx16 = 13, Xtpr = 14,
    Pdcm = 15, Dca = 18, Sse4_1 = 19, Sse4_2 = 20, X2Apic = 21, Movbe = 22,
    Popcnt = 23, TscDeadline = 24, Aes = 25, Xsave = 26, Osxsave = 27, Avx = 28,
    F16C = 29, Rdrnd = 30, Hypervisor = 31,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtendedFeature7Eb {
    Fsgsbase = 0, Ia32TsxAdjust = 1, Sgx = 2, Bmi1 = 3, Hle = 4, Avx2 = 5,
    FdpEx = 6, Smep = 7, Bmi2 = 8, Erms = 9, Invpcid = 10, Rtm = 11,
    Pqm = 12, Mpx = 14, Pqe = 15, Avx512f = 16, Avx512dq = 17, Rdseed = 18,
    Adx = 19, Smap = 20, Avx512ifma = 21, Pcommit = 22, Clflushopt = 23,
    Clwb = 24, IntelPt = 25, Avx512pf = 26, Avx512er = 27, Avx512cd = 28,
    Sha = 29, Avx512bw = 30, Avx512vl = 31,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtendedFeature7Ec {
    Prefetchwt1 = 0, Avx512vbmi = 1, Umip = 2, Pku = 3, Ospke = 4, Waitpkg = 5,
    Avx512vbmi2 = 6, CetSs = 7, Gfni = 8, Vaes = 9, Vpclmulqdq = 10,
    Avx512bitalg = 12, Avx512vpopcntdq = 14, Rdpid = 22,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtendedFeature7Ed {
    Lbrs = 0, Cldemote = 1, Mwaitu = 14,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtendedFeature80000001Ecx {
    LahfSahf = 0, CmpLegacy = 1, Svmm = 2, ExtApicSpace = 3, Cr8Legacy = 4,
    Abm = 5, Sse4a = 6, MisalignSse = 7, ThreeDNowPrefetch = 8, OsVw = 9,
    Ibs = 10, Xop = 11, Skinit = 12, Wdt = 13, Lwp = 15, Fma4 = 16,
    Tce = 17, NodeId = 19, Tbm = 21, TopologyExtensions = 22,
    PerfCtrExtCore = 23, PerfCtrExtNb = 24, BpExt = 26, Mwaitx = 29, XSaveOpt = 31,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtendedFeature80000001Edx {
    Fpu = 0, Vme = 1, De = 2, Pse = 3, Tsc = 4, Msr = 5, Pae = 6, Mce = 7,
    Cx8 = 8, Apic = 9, Syscall = 11, Mtrr = 12, Pge = 13, Mca = 14, Cmov = 15,
    Pat = 16, Pse36 = 17, Mmu = 20, Fxsr = 24, Mmx = 25, MmxExt = 26,
    Sse = 28, Sse2 = 29, Sse3 = 30, ThreeDNowExt = 31,
}

macro_rules! impl_from_enum {
    ($type:ty) => {
        impl From<$type> for u32 {
            fn from(val: $type) -> Self {
                val as u32
            }
        }
    };
}

impl_from_enum!(StandardFeatureEdx);
impl_from_enum!(ExtendedFeatureEcx);
impl_from_enum!(ExtendedFeature7Eb);
impl_from_enum!(ExtendedFeature7Ec);
impl_from_enum!(ExtendedFeature7Ed);
impl_from_enum!(ExtendedFeature80000001Ecx);
impl_from_enum!(ExtendedFeature80000001Edx);