use ntapi::ntexapi::KUSER_SHARED_DATA;

use crate::types::SystemTime;

pub struct KUserSharedData;

pub struct KUserSharedDataRef(&'static KUSER_SHARED_DATA);

impl KUserSharedData {
    const fn get() -> &'static KUSER_SHARED_DATA {
        unsafe { &*(0x7FFE0000 as *const KUSER_SHARED_DATA) }
    }

    pub const fn system_time() -> SystemTime {
        let time = Self::get().SystemTime;
        SystemTime(((time.High2Time as u64) << 32) | (time.LowPart as u64))
    }

    pub const fn suite_mask() -> u32 {
        Self::get().SuiteMask
    }

    pub const fn build_number() -> u32 {
        Self::get().NtBuildNumber
    }

    pub const fn product_type() -> u32 {
        Self::get().NtProductType
    }

    pub const fn major_version() -> u32 {
        Self::get().NtMajorVersion
    }
}

// STRUCT!{#[repr(packed(4))] struct KUSER_SHARED_DATA {
//     TickCountLowDeprecated: ULONG,
//     TickCountMultiplier: ULONG,
//     InterruptTime: KSYSTEM_TIME,
//     SystemTime: KSYSTEM_TIME,
//     TimeZoneBias: KSYSTEM_TIME,
//     ImageNumberLow: USHORT,
//     ImageNumberHigh: USHORT,
//     NtSystemRoot: [WCHAR; 260],
//     MaxStackTraceDepth: ULONG,
//     CryptoExponent: ULONG,
//     TimeZoneId: ULONG,
//     LargePageMinimum: ULONG,
//     AitSamplingValue: ULONG,
//     AppCompatFlag: ULONG,
//     RNGSeedVersion: ULONGLONG,
//     GlobalValidationRunlevel: ULONG,
//     TimeZoneBiasStamp: LONG,
//     NtBuildNumber: ULONG,
//     NtProductType: NT_PRODUCT_TYPE,
//     ProductTypeIsValid: BOOLEAN,
//     Reserved0: [UCHAR; 1],
//     NativeProcessorArchitecture: USHORT,
//     NtMajorVersion: ULONG,
//     NtMinorVersion: ULONG,
//     ProcessorFeatures: [BOOLEAN; PROCESSOR_FEATURE_MAX],
//     Reserved1: ULONG,
//     Reserved3: ULONG,
//     TimeSlip: ULONG,
//     AlternativeArchitecture: ALTERNATIVE_ARCHITECTURE_TYPE,
//     BootId: ULONG,
//     SystemExpirationDate: LARGE_INTEGER,
//     SuiteMask: ULONG,
//     KdDebuggerEnabled: BOOLEAN,
//     MitigationPolicies: UCHAR,
//     Reserved6: [UCHAR; 2],
//     ActiveConsoleId: ULONG,
//     DismountCount: ULONG,
//     ComPlusPackage: ULONG,
//     LastSystemRITEventTickCount: ULONG,
//     NumberOfPhysicalPages: ULONG,
//     SafeBootMode: BOOLEAN,
//     VirtualizationFlags: UCHAR,
//     Reserved12: [UCHAR; 2],
//     SharedDataFlags: ULONG,
//     DataFlagsPad: [ULONG; 1],
//     TestRetInstruction: ULONGLONG,
//     QpcFrequency: LONGLONG,
//     SystemCall: ULONG,
//     SystemCallPad0: ULONG,
//     SystemCallPad: [ULONGLONG; 2],
//     u: KUSER_SHARED_DATA_u,
//     //TickCountPad: [ULONG; 1],
//     Cookie: ULONG,
//     CookiePad: [ULONG; 1],
//     ConsoleSessionForegroundProcessId: LONGLONG,
//     TimeUpdateLock: ULONGLONG,
//     BaselineSystemTimeQpc: ULONGLONG,
//     BaselineInterruptTimeQpc: ULONGLONG,
//     QpcSystemTimeIncrement: ULONGLONG,
//     QpcInterruptTimeIncrement: ULONGLONG,
//     QpcSystemTimeIncrementShift: UCHAR,
//     QpcInterruptTimeIncrementShift: UCHAR,
//     UnparkedProcessorCount: USHORT,
//     EnclaveFeatureMask: [ULONG; 4],
//     TelemetryCoverageRound: ULONG,
//     UserModeGlobalLogger: [USHORT; 16],
//     ImageFileExecutionOptions: ULONG,
//     LangGenerationCount: ULONG,
//     Reserved4: ULONGLONG,
//     InterruptTimeBias: ULONG64,
//     QpcBias: ULONG64,
//     ActiveProcessorCount: ULONG,
//     ActiveGroupCount: UCHAR,
//     Reserved9: UCHAR,
//     QpcData: UCHAR,
//     TimeZoneBiasEffectiveStart: LARGE_INTEGER,
//     TimeZoneBiasEffectiveEnd: LARGE_INTEGER,
//     XState: XSTATE_CONFIGURATION,
// }}