use core::{fmt, ptr::{self, addr_of}};

use ntapi::ntexapi::{KUSER_SHARED_DATA, PROCESSOR_FEATURE_MAX};
use utils::U16CStackString;
use winapi::{shared::{basetsd::DWORD64, minwindef::DWORD}, um::winnt::{MAXIMUM_XSTATE_FEATURES, XSTATE_CONFIGURATION, XSTATE_FEATURE}};


#[repr(transparent)]
pub struct XStateConfiguration(pub &'static XSTATE_CONFIGURATION);

impl fmt::Display for XStateConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "XState:")?;
        writeln!(f, "  Enabled: {}", self.enabled_features_new())?;
        writeln!(f, "  Volatile: {}", self.enabled_volatile_features_new())?;
        writeln!(f, "  Supervisor: {}", self.enabled_supervisor_features_new())?;
        writeln!(f, "  Size: {}", self.size())?;
        writeln!(f, "  Control: {:#x}", self.control_flags())?;
        writeln!(f, "  AllSize: {}", self.all_feature_size())
    }
}

impl XStateConfiguration {
    pub fn new(xstate: &'static XSTATE_CONFIGURATION) -> Self {
        Self(xstate)
    }

    pub fn enabled_features(&self) -> DWORD64 {
        self.0.EnabledFeatures
    }

    pub fn enabled_features_new(&self) -> EnabledFeatures {
        EnabledFeatures::new(self.0.EnabledFeatures)
    }

    pub fn enabled_supervisor_features_new(&self) -> EnabledSupervisorFeatures {
        EnabledSupervisorFeatures::new(self.0.EnabledSupervisorFeatures)
    }

    pub fn enabled_features_display(&self) -> impl fmt::Display + '_ {
        struct EnabledFeatures<'a>(&'a XStateConfiguration);
        
        impl<'a> fmt::Display for EnabledFeatures<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let mut first = true;
                let features = [
                    (XStateFeature::LegacyFp, "Legacy FP"),
                    (XStateFeature::LegacySse, "Legacy SSE"),
                    (XStateFeature::Avx, "AVX"),
                    (XStateFeature::Bndregs, "BNDREGS"),
                    (XStateFeature::Bndcsr, "BNDCSR"),
                    (XStateFeature::Opmask, "OPMASK"),
                    (XStateFeature::ZmmHi256, "ZMM_HI256"),
                    (XStateFeature::Hi16Zmm, "HI16_ZMM"),
                    (XStateFeature::Lwp, "LWP"),
                    (XStateFeature::Int64, "INT64"),
                ];
                
                for (feature, name) in features {
                    if self.0.is_feature_enabled(feature) {
                        if !first {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", name)?;
                        first = false;
                    }
                }
                
                if first {
                    write!(f, "None")?;
                }
                
                Ok(())
            }
        }
        
        EnabledFeatures(self)
    }

    pub fn enabled_volatile_features_new(&self) -> EnabledVolatileFeatures {
        EnabledVolatileFeatures::new(self.0.EnabledVolatileFeatures)
    }

    pub fn size(&self) -> DWORD {
        self.0.Size
    }

    pub fn control_flags(&self) -> DWORD {
        self.0.ControlFlags
    }

    pub fn features(&self) -> XStateFeatures {
        unsafe {
            let ptr = addr_of!(self.0.Features);
            XStateFeatures(*ptr)
        }
    }

    pub fn enabled_supervisor_features(&self) -> DWORD64 {
        self.0.EnabledSupervisorFeatures
    }

    pub fn aligned_features(&self) -> DWORD64 {
        self.0.AlignedFeatures
    }

    pub fn all_feature_size(&self) -> DWORD {
        self.0.AllFeatureSize
    }

    pub fn all_features(&self) -> &[DWORD] {
        &self.0.AllFeatures
    }

    pub fn is_feature_enabled(&self, feature: XStateFeature) -> bool {
        (self.0.EnabledFeatures & (1 << feature as DWORD64)) != 0
    }

    pub fn is_volatile_feature_enabled(&self, feature: XStateFeature) -> bool {
        (self.0.EnabledVolatileFeatures & (1 << feature as DWORD64)) != 0
    }

    pub fn is_supervisor_feature_enabled(&self, feature: XStateFeature) -> bool {
        (self.0.EnabledSupervisorFeatures & (1 << feature as DWORD64)) != 0
    }
}

#[repr(transparent)]
pub struct EnabledVolatileFeatures(pub DWORD64);

impl EnabledVolatileFeatures {
    pub fn new(bits: DWORD64) -> Self {
        Self(bits)
    }

    pub fn is_enabled(&self, feature: XStateFeature) -> bool {
        (self.0 & (1 << feature as DWORD64)) != 0
    }

    pub fn iter(&self) -> EnabledVolatileFeaturesIter {
        EnabledVolatileFeaturesIter {
            bits: self.0,
            index: 0,
        }
    }
}

impl fmt::Display for EnabledVolatileFeatures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        let features = [
            (XStateFeature::LegacyFp, "Legacy FP"),
            (XStateFeature::LegacySse, "Legacy SSE"),
            (XStateFeature::Avx, "AVX"),
            (XStateFeature::Bndregs, "BNDREGS"),
            (XStateFeature::Bndcsr, "BNDCSR"),
            (XStateFeature::Opmask, "OPMASK"),
            (XStateFeature::ZmmHi256, "ZMM_HI256"),
            (XStateFeature::Hi16Zmm, "HI16_ZMM"),
            (XStateFeature::Lwp, "LWP"),
            (XStateFeature::Int64, "INT64"),
        ];
        
        for (feature, name) in features {
            if self.is_enabled(feature) {
                if !first {
                    write!(f, ", ")?;
                }
                write!(f, "{}", name)?;
                first = false;
            }
        }
        
        if first {
            write!(f, "None")?;
        }
        
        Ok(())
    }
}

impl fmt::Debug for EnabledVolatileFeatures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EnabledVolatileFeatures(0x{:x})", self.0)
    }
}

pub struct EnabledVolatileFeaturesIter {
    bits: DWORD64,
    index: u32,
}

impl Iterator for EnabledVolatileFeaturesIter {
    type Item = (XStateFeature, bool);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index <= 63 {
            let idx = self.index;
            self.index += 1;
            let feature = XStateFeature::from_u32(idx);
            let enabled = (self.bits & (1 << idx)) != 0;
            return Some((feature, enabled));
        }
        None
    }
}

#[repr(transparent)]
pub struct EnabledFeatures(pub DWORD64);

impl EnabledFeatures {
    pub fn new(bits: DWORD64) -> Self {
        Self(bits)
    }

    pub fn is_enabled(&self, feature: XStateFeature) -> bool {
        (self.0 & (1 << feature as DWORD64)) != 0
    }

    pub fn iter(&self) -> EnabledFeaturesIter {
        EnabledFeaturesIter {
            bits: self.0,
            index: 0,
        }
    }
}

impl fmt::Display for EnabledFeatures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        let features = [
            (XStateFeature::LegacyFp, "Legacy FP"),
            (XStateFeature::LegacySse, "Legacy SSE"),
            (XStateFeature::Avx, "AVX"),
            (XStateFeature::Bndregs, "BNDREGS"),
            (XStateFeature::Bndcsr, "BNDCSR"),
            (XStateFeature::Opmask, "OPMASK"),
            (XStateFeature::ZmmHi256, "ZMM_HI256"),
            (XStateFeature::Hi16Zmm, "HI16_ZMM"),
            (XStateFeature::Lwp, "LWP"),
            (XStateFeature::Int64, "INT64"),
        ];
        
        for (feature, name) in features {
            if self.is_enabled(feature) {
                if !first {
                    write!(f, ", ")?;
                }
                write!(f, "{}", name)?;
                first = false;
            }
        }
        
        if first {
            write!(f, "None")?;
        }
        
        Ok(())
    }
}

impl fmt::Debug for EnabledFeatures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EnabledFeatures(0x{:x})", self.0)
    }
}

pub struct EnabledFeaturesIter {
    bits: DWORD64,
    index: u32,
}

impl Iterator for EnabledFeaturesIter {
    type Item = (XStateFeature, bool);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index <= 63 {
            let idx = self.index;
            self.index += 1;
            let feature = XStateFeature::from_u32(idx);
            let enabled = (self.bits & (1 << idx)) != 0;
            return Some((feature, enabled));
        }
        None
    }
}

#[repr(transparent)]
pub struct EnabledSupervisorFeatures(pub DWORD64);

impl EnabledSupervisorFeatures {
    pub fn new(bits: DWORD64) -> Self {
        Self(bits)
    }

    pub fn is_enabled(&self, feature: SupervisorFeature) -> bool {
        (self.0 & (1 << feature as DWORD64)) != 0
    }

    pub fn iter(&self) -> EnabledSupervisorFeaturesIter {
        EnabledSupervisorFeaturesIter {
            bits: self.0,
            index: 0,
        }
    }
}

impl fmt::Display for EnabledSupervisorFeatures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        let features = [
            (SupervisorFeature::CET, "CET (Control-flow Enforcement Technology)"),
            (SupervisorFeature::PASID, "PASID (Process Address Space ID)"),
            (SupervisorFeature::CETUser, "CET_USER"),
            (SupervisorFeature::PT, "PT (Processor Trace)"),
            (SupervisorFeature::HAP, "HAP (Hypervisor Assist Page)"),
        ];
        
        for (feature, name) in features {
            if self.is_enabled(feature) {
                if !first {
                    write!(f, ", ")?;
                }
                write!(f, "{}", name)?;
                first = false;
            }
        }
        
        if first {
            write!(f, "None")?;
        }
        
        Ok(())
    }
}

impl fmt::Debug for EnabledSupervisorFeatures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EnabledSupervisorFeatures(0x{:x})", self.0)
    }
}

pub struct EnabledSupervisorFeaturesIter {
    bits: DWORD64,
    index: u32,
}

impl Iterator for EnabledSupervisorFeaturesIter {
    type Item = (SupervisorFeature, bool);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index <= 63 {
            let idx = self.index;
            self.index += 1;
            let feature = SupervisorFeature::from_u32(idx);
            let enabled = (self.bits & (1 << idx)) != 0;
            return Some((feature, enabled));
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SupervisorFeature {
    CET = 11,        // 0x800 - bit 11
    PASID = 12,      // 0x1000 - bit 12
    CETUser = 13,    // 0x2000 - bit 13
    PT = 14,         // 0x4000 - bit 14
    HAP = 15,        // 0x8000 - bit 15
}

impl SupervisorFeature {
    pub fn from_u32(val: u32) -> Self {
        match val {
            11 => SupervisorFeature::CET,
            12 => SupervisorFeature::PASID,
            13 => SupervisorFeature::CETUser,
            14 => SupervisorFeature::PT,
            15 => SupervisorFeature::HAP,
            _ => SupervisorFeature::CET, // fallback
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            SupervisorFeature::CET => "CET",
            SupervisorFeature::PASID => "PASID",
            SupervisorFeature::CETUser => "CET_USER",
            SupervisorFeature::PT => "PT",
            SupervisorFeature::HAP => "HAP",
        }
    }
}

impl fmt::Display for SupervisorFeature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum XStateFeature {
    LegacyFp = 0,
    LegacySse = 1,
    Avx = 2,
    Bndregs = 3,
    Bndcsr = 4,
    Opmask = 5,
    ZmmHi256 = 6,
    Hi16Zmm = 7,
    Lwp = 62,
    Int64 = 63, // XSTATE_MASK_INT64
}

impl fmt::Display for XStateFeature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XStateFeature::LegacyFp => write!(f, "LegacyFp"),
            XStateFeature::LegacySse => write!(f, "LegacySse"),
            XStateFeature::Avx => write!(f, "AVX"),
            XStateFeature::Bndregs => write!(f, "Bndregs"),
            XStateFeature::Bndcsr => write!(f, "Bndcsr"),
            XStateFeature::Opmask => write!(f, "Opmask"),
            XStateFeature::ZmmHi256 => write!(f, "ZmmHi256"),
            XStateFeature::Hi16Zmm => write!(f, "Hi16Zmm"),
            XStateFeature::Lwp => write!(f, "LWP"),
            XStateFeature::Int64 => write!(f, "Int64"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeZoneId {
    Unknown = 0,
    Standard = 1,
    Daylight = 2,
}

impl XStateFeature {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => XStateFeature::LegacyFp,
            1 => XStateFeature::LegacySse,
            2 => XStateFeature::Avx,
            3 => XStateFeature::Bndregs,
            4 => XStateFeature::Bndcsr,
            5 => XStateFeature::Opmask,
            6 => XStateFeature::ZmmHi256,
            7 => XStateFeature::Hi16Zmm,
            62 => XStateFeature::Lwp,
            63 => XStateFeature::Int64,
            _ => XStateFeature::LegacyFp, // fallback
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NtProductType {
    WinNt = 1,
    LanManNt = 2,
    Server = 3,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessorArchitecture {
    Intel = 0,
    Arm = 5,
    Amd64 = 9,
    Arm64 = 12,
    Unknown = 0xffff,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlternativeArchitecture {
    Standard = 0,
    NEC = 1,
    PowerPc = 2,
    Alpha = 3,
    Mips = 4,
}

#[repr(transparent)]
pub struct XStateFeatureEntry(pub XSTATE_FEATURE);

impl XStateFeatureEntry {
    pub fn offset(&self) -> DWORD {
        self.0.Offset
    }

    pub fn size(&self) -> DWORD {
        self.0.Size
    }
}

impl fmt::Display for XStateFeatureEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Offset: {}, Size: {}", self.offset(), self.size())
    }
}

impl fmt::Debug for XStateFeatureEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("XStateFeatureEntry")
            .field("Offset", &self.offset())
            .field("Size", &self.size())
            .finish()
    }
}

#[repr(transparent)]
pub struct XStateFeatures([XSTATE_FEATURE; MAXIMUM_XSTATE_FEATURES]);

impl XStateFeatures {
    pub fn get(&self, index: usize) -> Option<XStateFeatureEntry> {
        if index < self.0.len() {
            Some(XStateFeatureEntry(self.0[index]))
        } else {
            None
        }
    }

    pub fn get_by_feature(&self, feature: XStateFeature) -> Option<XStateFeatureEntry> {
        self.get(feature as usize)
    }

    pub fn iter(&self) -> XStateFeaturesIter {
        XStateFeaturesIter {
            features: self,
            index: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl AsRef<[XSTATE_FEATURE]> for XStateFeatures {
    fn as_ref(&self) -> &[XSTATE_FEATURE] {
        &self.0
    }
}

pub struct XStateFeaturesIter<'a> {
    features: &'a XStateFeatures,
    index: usize,
}

impl<'a> Iterator for XStateFeaturesIter<'a> {
    type Item = (XStateFeature, XStateFeatureEntry);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.features.len() {
            return None;
        }

        let idx = self.index;
        self.index += 1;

        Some((
            XStateFeature::from_u32(idx as u32),
            XStateFeatureEntry(self.features.0[idx]),
        ))
    }
}

impl fmt::Display for XStateFeatures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, (feature, entry)) in self.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", feature, entry)?;
        }
        write!(f, "]")
    }
}

