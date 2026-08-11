use ntapi::{ntpebteb::TEB, winapi_local::um::winnt::NtCurrentTeb};

use crate::Environment;

pub struct ThreadEnvironmentBlock(*mut TEB);

impl core::ops::Deref for ThreadEnvironmentBlock {
    type Target = TEB;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}


impl ThreadEnvironmentBlock {
    pub fn current_process() -> Self {
        unsafe {
            let teb: *mut TEB = NtCurrentTeb();
            Self(teb)
        }
    }

    pub fn last_error(&self) -> u32 {
        unsafe { (*self.0).LastErrorValue }
    }

    pub fn last_status(&self) -> i32 {
        unsafe { (*self.0).LastStatusValue }
    }

    pub fn environment(&self) -> Environment {
        Environment(unsafe { (*self.0).EnvironmentPointer })
    }

    pub fn is_impersonating(&self) -> bool {
        let is_impersonating = unsafe { (*self.0).IsImpersonating };
        is_impersonating == 1
    }
}

/*
 NtTib: NT_TIB,
    EnvironmentPointer: PVOID,
    ClientId: CLIENT_ID,
    ActiveRpcHandle: PVOID,
    ThreadLocalStoragePointer: PVOID,
    ProcessEnvironmentBlock: PPEB,
    LastErrorValue: ULONG,
    CountOfOwnedCriticalSections: ULONG,
    CsrClientThread: PVOID,
    Win32ThreadInfo: PVOID,
    User32Reserved: [ULONG; 26],
    UserReserved: [ULONG; 5],
    WOW32Reserved: PVOID,
    CurrentLocale: LCID,
    FpSoftwareStatusRegister: ULONG,
    ReservedForDebuggerInstrumentation: [PVOID; 16],
    SystemReserved1: [PVOID; 30],
    PlaceholderCompatibilityMode: CHAR,
    PlaceholderReserved: [CHAR; 11],
    ProxiedProcessId: ULONG,
    ActivationStack: ACTIVATION_CONTEXT_STACK,
    WorkingOnBehalfTicket: [UCHAR; 8],
    ExceptionCode: NTSTATUS,
    ActivationContextStackPointer: PACTIVATION_CONTEXT_STACK,
    InstrumentationCallbackSp: ULONG_PTR,
    InstrumentationCallbackPreviousPc: ULONG_PTR,
    InstrumentationCallbackPreviousSp: ULONG_PTR,
    TxFsContext: ULONG,
    InstrumentationCallbackDisabled: BOOLEAN,
    GdiTebBatch: GDI_TEB_BATCH,
    RealClientId: CLIENT_ID,
    GdiCachedProcessHandle: HANDLE,
    GdiClientPID: ULONG,
    GdiClientTID: ULONG,
    GdiThreadLocalInfo: PVOID,
    Win32ClientInfo: [ULONG_PTR; 62],
    glDispatchTable: [PVOID; 233],
    glReserved1: [ULONG_PTR; 29],
    glReserved2: PVOID,
    glSectionInfo: PVOID,
    glSection: PVOID,
    glTable: PVOID,
    glCurrentRC: PVOID,
    glContext: PVOID,
    LastStatusValue: NTSTATUS,
    StaticUnicodeString: UNICODE_STRING,
    StaticUnicodeBuffer: [WCHAR; 261],
    DeallocationStack: PVOID,
    TlsSlots: [PVOID; 64],
    TlsLinks: LIST_ENTRY,
    Vdm: PVOID,
    ReservedForNtRpc: PVOID,
    DbgSsReserved: [PVOID; 2],
    HardErrorMode: ULONG,
    Instrumentation: [PVOID; 11],
    ActivityId: GUID,
    SubProcessTag: PVOID,
    PerflibData: PVOID,
    EtwTraceData: PVOID,
    WinSockData: PVOID,
    GdiBatchCount: ULONG,
    u: TEB_u,
    GuaranteedStackBytes: ULONG,
    ReservedForPerf: PVOID,
    ReservedForOle: PVOID,
    WaitingOnLoaderLock: ULONG,
    SavedPriorityState: PVOID,
    ReservedForCodeCoverage: ULONG_PTR,
    ThreadPoolData: PVOID,
    TlsExpansionSlots: *mut PVOID,
    DeallocationBStore: PVOID,
    BStoreLimit: PVOID,
    MuiGeneration: ULONG,
    IsImpersonating: ULONG,
    NlsCache: PVOID,
    pShimData: PVOID,
    HeapVirtualAffinity: USHORT,
    LowFragHeapDataSlot: USHORT,
    CurrentTransactionHandle: HANDLE,
    ActiveFrame: PTEB_ACTIVE_FRAME,
    FlsData: PVOID,
    PreferredLanguages: PVOID,
    UserPrefLanguages: PVOID,
    MergedPrefLanguages: PVOID,
    MuiImpersonation: ULONG,
    CrossTebFlags: USHORT,
    SameTebFlags: USHORT,
    TxnScopeEnterCallback: PVOID,
    TxnScopeExitCallback: PVOID,
    TxnScopeContext: PVOID,
    LockCount: ULONG,
    WowTebOffset: LONG,
    ResourceRetValue: PVOID,
    ReservedForWdf: PVOID,
    ReservedForCrt: ULONGLONG,
    EffectiveContainerId: GUID,
*/