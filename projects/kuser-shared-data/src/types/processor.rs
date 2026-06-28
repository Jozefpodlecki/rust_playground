use core::fmt;

use ntapi::ntexapi::PROCESSOR_FEATURE_MAX;

#[repr(transparent)]
pub struct ProcessorFeatures([u8; PROCESSOR_FEATURE_MAX]);

impl ProcessorFeatures {
    pub fn new(features: &[u8]) -> Self {
        let mut arr = [0u8; PROCESSOR_FEATURE_MAX];
        let len = features.len().min(PROCESSOR_FEATURE_MAX);
        arr[..len].copy_from_slice(&features[..len]);
        Self(arr)
    }

    pub fn get(&self, feature: ProcessorFeature) -> bool {
        self.0[feature as usize] != 0
    }

    pub fn iter(&self) -> ProcessorFeaturesIter {
        ProcessorFeaturesIter {
            features: self,
            index: 0,
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Display for ProcessorFeatures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        let mut count = 0;
        
        for (feature, enabled) in self.iter() {
            if enabled {
                if !first {
                    if count % 5 == 0 {
                        writeln!(f)?;
                        write!(f, "  ")?;
                    } else {
                        write!(f, ", ")?;
                    }
                } else {
                    write!(f, "  ")?;
                }
                write!(f, "{}", feature)?;
                first = false;
                count += 1;
            }
        }
        
        if first {
            write!(f, "None")?;
        }
        
        Ok(())
    }
}

pub struct ProcessorFeaturesIter<'a> {
    features: &'a ProcessorFeatures,
    index: usize,
}

impl<'a> Iterator for ProcessorFeaturesIter<'a> {
    type Item = (ProcessorFeature, bool);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= PROCESSOR_FEATURE_MAX {
            return None;
        }
        let idx = self.index;
        self.index += 1;
        Some((ProcessorFeature::from_u32(idx as u32), self.features.0[idx] != 0))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ProcessorFeature {
    Fpu = 0,
    Vme = 1,
    De = 2,
    Pse = 3,
    Tsc = 4,
    Msr = 5,
    Pae = 6,
    Mce = 7,
    Cx8 = 8,
    Apic = 9,
    Sep = 10,
    Mtrr = 11,
    Pge = 12,
    Mca = 13,
    Cmov = 14,
    Pat = 15,
    Pse36 = 16,
    Psn = 17,
    Clf = 18,
    Dtes = 19,
    Acpi = 20,
    Mmx = 21,
    Fxsr = 22,
    Sse = 23,
    Sse2 = 24,
    Ss = 25,
    Htt = 26,
    Tm = 27,
    Pbe = 28,
    FxsrOpt = 29,
    Pdpe1Gb = 30,
    Rdtscp = 31,
    Lm = 32,
    ThreeDNowExt = 33,
    ThreeDNow = 34,
    Smp = 35,
    Nx = 36,
    MmxExt = 37,
    Longrun = 38,
    LongrunMsr = 39,
    Svm = 40,
    EmmX = 41,
    TscPause = 42,
    IntelTbt = 43,
    IntelHtt = 44,
    IntelMp = 45,
    IntelVmx = 46,
    IntelSmx = 47,
    IntelEst = 48,
    IntelTm2 = 49,
    IntelCid = 50,
    IntelCx16 = 51,
    IntelXtpr = 52,
    IntelDca = 53,
    IntelSse41 = 54,
    IntelSse42 = 55,
    IntelPopcnt = 56,
    IntelAes = 57,
    IntelPclmulqdq = 58,
    IntelXsave = 59,
    IntelOsxsave = 60,
    IntelAvx = 61,
    IntelF16c = 62,
}

impl ProcessorFeature {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => ProcessorFeature::Fpu,
            1 => ProcessorFeature::Vme,
            2 => ProcessorFeature::De,
            3 => ProcessorFeature::Pse,
            4 => ProcessorFeature::Tsc,
            5 => ProcessorFeature::Msr,
            6 => ProcessorFeature::Pae,
            7 => ProcessorFeature::Mce,
            8 => ProcessorFeature::Cx8,
            9 => ProcessorFeature::Apic,
            10 => ProcessorFeature::Sep,
            11 => ProcessorFeature::Mtrr,
            12 => ProcessorFeature::Pge,
            13 => ProcessorFeature::Mca,
            14 => ProcessorFeature::Cmov,
            15 => ProcessorFeature::Pat,
            16 => ProcessorFeature::Pse36,
            17 => ProcessorFeature::Psn,
            18 => ProcessorFeature::Clf,
            19 => ProcessorFeature::Dtes,
            20 => ProcessorFeature::Acpi,
            21 => ProcessorFeature::Mmx,
            22 => ProcessorFeature::Fxsr,
            23 => ProcessorFeature::Sse,
            24 => ProcessorFeature::Sse2,
            25 => ProcessorFeature::Ss,
            26 => ProcessorFeature::Htt,
            27 => ProcessorFeature::Tm,
            28 => ProcessorFeature::Pbe,
            29 => ProcessorFeature::FxsrOpt,
            30 => ProcessorFeature::Pdpe1Gb,
            31 => ProcessorFeature::Rdtscp,
            32 => ProcessorFeature::Lm,
            33 => ProcessorFeature::ThreeDNowExt,
            34 => ProcessorFeature::ThreeDNow,
            35 => ProcessorFeature::Smp,
            36 => ProcessorFeature::Nx,
            37 => ProcessorFeature::MmxExt,
            38 => ProcessorFeature::Longrun,
            39 => ProcessorFeature::LongrunMsr,
            40 => ProcessorFeature::Svm,
            41 => ProcessorFeature::EmmX,
            42 => ProcessorFeature::TscPause,
            43 => ProcessorFeature::IntelTbt,
            44 => ProcessorFeature::IntelHtt,
            45 => ProcessorFeature::IntelMp,
            46 => ProcessorFeature::IntelVmx,
            47 => ProcessorFeature::IntelSmx,
            48 => ProcessorFeature::IntelEst,
            49 => ProcessorFeature::IntelTm2,
            50 => ProcessorFeature::IntelCid,
            51 => ProcessorFeature::IntelCx16,
            52 => ProcessorFeature::IntelXtpr,
            53 => ProcessorFeature::IntelDca,
            54 => ProcessorFeature::IntelSse41,
            55 => ProcessorFeature::IntelSse42,
            56 => ProcessorFeature::IntelPopcnt,
            57 => ProcessorFeature::IntelAes,
            58 => ProcessorFeature::IntelPclmulqdq,
            59 => ProcessorFeature::IntelXsave,
            60 => ProcessorFeature::IntelOsxsave,
            61 => ProcessorFeature::IntelAvx,
            62 => ProcessorFeature::IntelF16c,
            _ => ProcessorFeature::Fpu,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ProcessorFeature::Fpu => "FPU",
            ProcessorFeature::Vme => "VME",
            ProcessorFeature::De => "DE",
            ProcessorFeature::Pse => "PSE",
            ProcessorFeature::Tsc => "TSC",
            ProcessorFeature::Msr => "MSR",
            ProcessorFeature::Pae => "PAE",
            ProcessorFeature::Mce => "MCE",
            ProcessorFeature::Cx8 => "CX8",
            ProcessorFeature::Apic => "APIC",
            ProcessorFeature::Sep => "SEP",
            ProcessorFeature::Mtrr => "MTRR",
            ProcessorFeature::Pge => "PGE",
            ProcessorFeature::Mca => "MCA",
            ProcessorFeature::Cmov => "CMOV",
            ProcessorFeature::Pat => "PAT",
            ProcessorFeature::Pse36 => "PSE36",
            ProcessorFeature::Psn => "PSN",
            ProcessorFeature::Clf => "CLF",
            ProcessorFeature::Dtes => "DTES",
            ProcessorFeature::Acpi => "ACPI",
            ProcessorFeature::Mmx => "MMX",
            ProcessorFeature::Fxsr => "FXSR",
            ProcessorFeature::Sse => "SSE",
            ProcessorFeature::Sse2 => "SSE2",
            ProcessorFeature::Ss => "SS",
            ProcessorFeature::Htt => "HTT",
            ProcessorFeature::Tm => "TM",
            ProcessorFeature::Pbe => "PBE",
            ProcessorFeature::FxsrOpt => "FXSR_OPT",
            ProcessorFeature::Pdpe1Gb => "PDPE1GB",
            ProcessorFeature::Rdtscp => "RDTSCP",
            ProcessorFeature::Lm => "LM",
            ProcessorFeature::ThreeDNowExt => "3DNOWEXT",
            ProcessorFeature::ThreeDNow => "3DNOW",
            ProcessorFeature::Smp => "SMP",
            ProcessorFeature::Nx => "NX",
            ProcessorFeature::MmxExt => "MMXEXT",
            ProcessorFeature::Longrun => "LONGRUN",
            ProcessorFeature::LongrunMsr => "LONGRUN_MSR",
            ProcessorFeature::Svm => "SVM",
            ProcessorFeature::EmmX => "EMMX",
            ProcessorFeature::TscPause => "TSC_PAUSE",
            ProcessorFeature::IntelTbt => "INTEL_TBT",
            ProcessorFeature::IntelHtt => "INTEL_HTT",
            ProcessorFeature::IntelMp => "INTEL_MP",
            ProcessorFeature::IntelVmx => "INTEL_VMX",
            ProcessorFeature::IntelSmx => "INTEL_SMX",
            ProcessorFeature::IntelEst => "INTEL_EST",
            ProcessorFeature::IntelTm2 => "INTEL_TM2",
            ProcessorFeature::IntelCid => "INTEL_CID",
            ProcessorFeature::IntelCx16 => "INTEL_CX16",
            ProcessorFeature::IntelXtpr => "INTEL_XTPR",
            ProcessorFeature::IntelDca => "INTEL_DCA",
            ProcessorFeature::IntelSse41 => "INTEL_SSE4_1",
            ProcessorFeature::IntelSse42 => "INTEL_SSE4_2",
            ProcessorFeature::IntelPopcnt => "INTEL_POPCNT",
            ProcessorFeature::IntelAes => "INTEL_AES",
            ProcessorFeature::IntelPclmulqdq => "INTEL_PCLMULQDQ",
            ProcessorFeature::IntelXsave => "INTEL_XSAVE",
            ProcessorFeature::IntelOsxsave => "INTEL_OSXSAVE",
            ProcessorFeature::IntelAvx => "INTEL_AVX",
            ProcessorFeature::IntelF16c => "INTEL_F16C",
        }
    }
}

impl fmt::Display for ProcessorFeature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

