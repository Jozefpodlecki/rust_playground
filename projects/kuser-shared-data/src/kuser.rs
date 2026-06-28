use core::{fmt, ptr::{self, addr_of}};

use ntapi::ntexapi::KUSER_SHARED_DATA;
use utils::U16CStackString;

use crate::types::*;

#[repr(transparent)]
pub struct KUserSharedData(KUSER_SHARED_DATA);

impl fmt::Display for KUserSharedData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== KUSER_SHARED_DATA ===")?;
        writeln!(f, "System Root: {}", self.nt_system_root())?;
        writeln!(f, "System Time: {} (100ns intervals)", self.system_time())?;
        writeln!(f, "Interrupt Time: {}", self.interrupt_time())?;
        writeln!(f, "Time Zone Bias: {}", self.time_zone_bias())?;
        writeln!(f, "Tick Count Multiplier: {}", self.tick_count_multiplier())?;
        writeln!(f, "NT Version: {}", self.nt_version())?;
        writeln!(f, "NT Product Type: {:?}", self.nt_product_type())?;
        writeln!(f, "Processor Architecture: {:?}", self.native_processor_architecture())?;
        writeln!(f, "Active Processor Count: {}", self.active_processor_count())?;
        writeln!(f, "Physical Memory: {}", self.physical_pages())?;
        writeln!(f, "Boot ID: {}", self.boot_id())?;
        writeln!(f, "Debugger Present: {}", self.is_debugger_present())?;
        writeln!(f, "Safe Boot Mode: {}", self.safe_boot_mode())?;
        writeln!(f, "Time Zone ID: {:?}", self.time_zone_id())?;
        writeln!(f, "System Expiration Date: {}", self.system_expiration_date())?;
        writeln!(f, "Suite Mask: {}", self.suite_mask())?;
        writeln!(f, "Active Console ID: {}", self.active_console_id())?;
        writeln!(f, "Cookie: {:#x}", self.cookie())?;
        writeln!(f, "QPC Frequency: {}", self.qpc_frequency())?;
        writeln!(f, "System Call: {:#x}", self.system_call())?;
        writeln!(f, "Shared Data Flags: {}", self.shared_data_flags())?;
        writeln!(f, "Mitigation Policies: {}", self.mitigation_policies())?;
        
        writeln!(f, "XState: {}", self.xstate())?;
        writeln!(f, "Processor Features: {}", self.processor_features())?;
        
        Ok(())
    }
}

impl KUserSharedData {
    pub fn new() -> &'static Self {
        unsafe {
            &*(0x7FFE0000 as *const Self)
        }
    }
     
    pub fn nt_system_root(&self) -> U16CStackString<100> {
        U16CStackString::<100>::from_ptr(self.0.NtSystemRoot.as_ptr()).unwrap()
    }

    pub fn system_time(&self) -> SystemTime {
        let time = &self.0.SystemTime;
        SystemTime(((time.High2Time as u64) << 32) | (time.LowPart as u64))
    }

    pub fn interrupt_time(&self) -> SystemTime {
        let time = &self.0.InterruptTime;
        SystemTime(((time.High2Time as u64) << 32) | (time.LowPart as u64))
    }

    pub fn time_zone_bias(&self) -> SystemTime {
        let time = &self.0.TimeZoneBias;
        SystemTime(((time.High2Time as u64) << 32) | (time.LowPart as u64))
    }

    pub fn tick_count_multiplier(&self) -> u32 {
        self.0.TickCountMultiplier
    }

    pub fn nt_major_version(&self) -> u32 {
        self.0.NtMajorVersion
    }

    pub fn nt_minor_version(&self) -> u32 {
        self.0.NtMinorVersion
    }

    pub fn nt_build_number(&self) -> u32 {
        self.0.NtBuildNumber
    }

    pub fn nt_version(&self) -> NtVersion {
        NtVersion {
            major: self.nt_major_version(),
            minor: self.nt_minor_version(),
            build: self.nt_build_number(),
        }
    }

    pub fn is_debugger_present(&self) -> bool {
        self.0.KdDebuggerEnabled != 0
    }

    pub fn tick_count_low_deprecated(&self) -> u32 {
        self.0.TickCountLowDeprecated
    }

    pub fn image_number(&self) -> u32 {
        (self.0.ImageNumberHigh as u32) << 16 | (self.0.ImageNumberLow as u32)
    }

    pub fn max_stack_trace_depth(&self) -> u32 {
        self.0.MaxStackTraceDepth
    }

    pub fn crypto_exponent(&self) -> u32 {
        self.0.CryptoExponent
    }

    pub fn time_zone_id(&self) -> TimeZoneId {
        match self.0.TimeZoneId {
            0 => TimeZoneId::Unknown,
            1 => TimeZoneId::Standard,
            2 => TimeZoneId::Daylight,
            _ => TimeZoneId::Unknown,
        }
    }

    pub fn large_page_minimum(&self) -> u32 {
        self.0.LargePageMinimum
    }

    pub fn ait_sampling_value(&self) -> u32 {
        self.0.AitSamplingValue
    }

    pub fn app_compat_flag(&self) -> u32 {
        self.0.AppCompatFlag
    }

    pub fn rng_seed_version(&self) -> u64 {
        self.0.RNGSeedVersion
    }

    pub fn global_validation_runlevel(&self) -> u32 {
        self.0.GlobalValidationRunlevel
    }

    pub fn time_zone_bias_stamp(&self) -> i32 {
        self.0.TimeZoneBiasStamp
    }

    pub fn nt_product_type(&self) -> NtProductType {
        match self.0.NtProductType {
            1 => NtProductType::WinNt,
            2 => NtProductType::LanManNt,
            3 => NtProductType::Server,
            _ => NtProductType::Unknown,
        }
    }

    pub fn product_type_is_valid(&self) -> bool {
        self.0.ProductTypeIsValid != 0
    }

    pub fn native_processor_architecture(&self) -> ProcessorArchitecture {
        match self.0.NativeProcessorArchitecture {
            0 => ProcessorArchitecture::Intel,
            5 => ProcessorArchitecture::Arm,
            9 => ProcessorArchitecture::Amd64,
            12 => ProcessorArchitecture::Arm64,
            0xffff => ProcessorArchitecture::Unknown,
            _ => ProcessorArchitecture::Unknown,
        }
    }

    pub fn processor_features(&self) -> ProcessorFeatures {
        ProcessorFeatures::new(&self.0.ProcessorFeatures)
    }

    pub fn time_slip(&self) -> u32 {
        self.0.TimeSlip
    }

    pub fn alternative_architecture(&self) -> AlternativeArchitecture {
        match self.0.AlternativeArchitecture {
            0 => AlternativeArchitecture::Standard,
            1 => AlternativeArchitecture::NEC,
            2 => AlternativeArchitecture::PowerPc,
            3 => AlternativeArchitecture::Alpha,
            4 => AlternativeArchitecture::Mips,
            _ => AlternativeArchitecture::Standard,
        }
    }

    pub fn boot_id(&self) -> u32 {
        self.0.BootId
    }

    pub fn system_expiration_date(&self) -> i64 {
        unsafe {
            let val = ptr::read_unaligned(addr_of!(self.0.SystemExpirationDate));
            *val.QuadPart()
        }
    }

    pub fn suite_mask(&self) -> SuiteMask {
        SuiteMask(self.0.SuiteMask)
    }

    pub fn mitigation_policies(&self) -> MitigationPolicies {
        MitigationPolicies(self.0.MitigationPolicies)
    }

    pub fn active_console_id(&self) -> u32 {
        self.0.ActiveConsoleId
    }

    pub fn dismount_count(&self) -> u32 {
        self.0.DismountCount
    }

    pub fn com_plus_package(&self) -> u32 {
        self.0.ComPlusPackage
    }

    pub fn last_system_rit_event_tick_count(&self) -> u32 {
        self.0.LastSystemRITEventTickCount
    }

    pub fn physical_pages(&self) -> PhysicalPages {
        PhysicalPages(self.0.NumberOfPhysicalPages)
    }

    pub fn safe_boot_mode(&self) -> bool {
        self.0.SafeBootMode != 0
    }

    pub fn virtualization_flags(&self) -> u8 {
        self.0.VirtualizationFlags
    }

    pub fn shared_data_flags(&self) -> SharedDataFlags {
        SharedDataFlags(self.0.SharedDataFlags)
    }

    pub fn test_ret_instruction(&self) -> u64 {
        self.0.TestRetInstruction
    }

    pub fn qpc_frequency(&self) -> i64 {
        self.0.QpcFrequency
    }

    pub fn system_call(&self) -> u32 {
        self.0.SystemCall
    }

    pub fn cookie(&self) -> u32 {
        self.0.Cookie
    }

    pub fn console_session_foreground_process_id(&self) -> i64 {
        self.0.ConsoleSessionForegroundProcessId
    }

    pub fn time_update_lock(&self) -> u64 {
        self.0.TimeUpdateLock
    }

    pub fn baseline_system_time_qpc(&self) -> u64 {
        self.0.BaselineSystemTimeQpc
    }

    pub fn baseline_interrupt_time_qpc(&self) -> u64 {
        self.0.BaselineInterruptTimeQpc
    }

    pub fn qpc_system_time_increment(&self) -> u64 {
        self.0.QpcSystemTimeIncrement
    }

    pub fn qpc_interrupt_time_increment(&self) -> u64 {
        self.0.QpcInterruptTimeIncrement
    }

    pub fn qpc_system_time_increment_shift(&self) -> u8 {
        self.0.QpcSystemTimeIncrementShift
    }

    pub fn qpc_interrupt_time_increment_shift(&self) -> u8 {
        self.0.QpcInterruptTimeIncrementShift
    }

    pub fn unparked_processor_count(&self) -> u16 {
        self.0.UnparkedProcessorCount
    }

    pub fn enclave_feature_mask(&self) -> &[u32] {
        &self.0.EnclaveFeatureMask
    }

    pub fn telemetry_coverage_round(&self) -> u32 {
        self.0.TelemetryCoverageRound
    }

    pub fn user_mode_global_logger(&self) -> &[u16] {
        &self.0.UserModeGlobalLogger
    }

    pub fn image_file_execution_options(&self) -> u32 {
        self.0.ImageFileExecutionOptions
    }

    pub fn lang_generation_count(&self) -> u32 {
        self.0.LangGenerationCount
    }

    pub fn interrupt_time_bias(&self) -> u64 {
        self.0.InterruptTimeBias
    }

    pub fn qpc_bias(&self) -> u64 {
        self.0.QpcBias
    }

    pub fn active_processor_count(&self) -> u32 {
        self.0.ActiveProcessorCount
    }

    pub fn active_group_count(&self) -> u8 {
        self.0.ActiveGroupCount
    }

    pub fn qpc_data(&self) -> QpcData {
        QpcData(self.0.QpcData)
    }

    pub fn time_zone_bias_effective_start(&self) -> i64 {
        unsafe {
            let val = ptr::read_unaligned(addr_of!(self.0.TimeZoneBiasEffectiveStart));
            *val.QuadPart()
        }
    }

    pub fn time_zone_bias_effective_end(&self) -> i64 {
        unsafe {
            let val = ptr::read_unaligned(addr_of!(self.0.TimeZoneBiasEffectiveEnd));
            *val.QuadPart()
        }
    }

    pub fn xstate(&self) -> XStateConfiguration {
        unsafe {
            let ptr = addr_of!(self.0.XState);
            XStateConfiguration(&*ptr)
        }
    }
}