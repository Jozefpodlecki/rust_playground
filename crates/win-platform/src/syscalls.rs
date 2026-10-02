use core::arch::naked_asm;
use ntapi::{ntapi_base::PCLIENT_ID, ntexapi::SYSTEM_INFORMATION_CLASS, ntioapi::{FILE_INFORMATION_CLASS, PIO_APC_ROUTINE, PIO_STATUS_BLOCK}, ntmmapi::{MEMORY_INFORMATION_CLASS, SECTION_INFORMATION_CLASS, SECTION_INHERIT}, ntobapi::OBJECT_INFORMATION_CLASS, ntpsapi::{PPS_APC_ROUTINE, PPS_ATTRIBUTE_LIST, PPS_CREATE_INFO, PROCESSINFOCLASS, THREADINFOCLASS}};
use winapi::{shared::{basetsd::{SIZE_T, ULONG_PTR}, minwindef::{PULONG, ULONG}, ntdef::{BOOLEAN, EVENT_TYPE, PLARGE_INTEGER, PLONG, POBJECT_ATTRIBUTES, PUNICODE_STRING}}, um::winnt::{ACCESS_MASK, PCONTEXT}};

use crate::{NtStatus, types::{HANDLE, PCVOID, PHANDLE, PKContinueArgument, PVOID}};

#[unsafe(naked)]
pub extern "system" fn NtFreeVirtualMemory(
    ProcessHandle: HANDLE,
    BaseAddress: *mut PVOID,
    RegionSize: *mut usize,
    FreeType: u32,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x1E",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtCreateMutant(
    MutantHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    InitialOwner: BOOLEAN,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0xBA",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtReleaseMutant(
    MutantHandle: HANDLE,
    PreviousCount: PLONG,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x20",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtResumeProcess(ProcessHandle: HANDLE) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x18A",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtGetContextThread(
    ThreadHandle: HANDLE,
    ThreadContext: PCONTEXT,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0xFB",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtSetContextThread(
    ThreadHandle: HANDLE,
    ThreadContext: PCONTEXT,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x19A",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtQueueApcThread(
    ThreadHandle: HANDLE,
    ApcRoutine: PVOID,
    ApcArgument1: PVOID,
    ApcArgument2: PVOID,
    ApcArgument3: PVOID,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x45",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtOpenThread(
    ThreadHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    ClientId: PCLIENT_ID,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x139",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtResumeThread(
    ThreadHandle: HANDLE,
    PreviousSuspendCount: PULONG,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x52",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtQueryObject(
    Handle: HANDLE,
    ObjectInformationClass: OBJECT_INFORMATION_CLASS,
    ObjectInformation: PVOID,
    ObjectInformationLength: u32,
    ReturnLength: PULONG,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x10",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtOpenProcess(
    ProcessHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    ClientId: PCLIENT_ID,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x26",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtContinueEx(
    ContextRecord: PCONTEXT,
    ContinueArgument: PKContinueArgument,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0xA5",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtContinue(
    ContextRecord: PCONTEXT,
    TestAlert: BOOLEAN,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x43",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtClose(
    Handle: HANDLE,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0xF",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtQueryInformationProcess(
    ProcessHandle: HANDLE,
    ProcessInformationClass: PROCESSINFOCLASS,
    ProcessInformation: PVOID,
    ProcessInformationLength: u32,
    ReturnLength: PULONG,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x19",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtQuerySystemInformation(
    SystemInformationClass: SYSTEM_INFORMATION_CLASS,
    SystemInformation: PVOID,
    SystemInformationLength: u32,
    ReturnLength: PULONG,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x36",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtQueryInformationFile(
    FileHandle: HANDLE,
    IoStatusBlock: PIO_STATUS_BLOCK,
    FileInformation: PVOID,
    Length: u32,
    FileInformationClass: FILE_INFORMATION_CLASS,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x11",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtQueryInformationThread(
    ThreadHandle: HANDLE,
    ThreadInformationClass: THREADINFOCLASS,
    ThreadInformation: PVOID,
    ThreadInformationLength: u32,
    ReturnLength: PULONG,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x25",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtSetInformationFile(
    FileHandle: HANDLE,
    IoStatusBlock: PIO_STATUS_BLOCK,
    FileInformation: PVOID,
    Length: u32,
    FileInformationClass: FILE_INFORMATION_CLASS,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x27",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtQueryDirectoryFile(
    FileHandle: HANDLE,
    Event: HANDLE,
    ApcRoutine: PIO_APC_ROUTINE,
    ApcContext: PVOID,
    IoStatusBlock: PIO_STATUS_BLOCK,
    FileInformation: PVOID,
    Length: u32,
    FileInformationClass: FILE_INFORMATION_CLASS,
    ReturnSingleEntry: BOOLEAN,
    FileName: PUNICODE_STRING,
    RestartScan: BOOLEAN,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x35",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub fn NtFsControlFile(
    FileHandle: HANDLE,
    Event: HANDLE,
    ApcRoutine: PIO_APC_ROUTINE,
    ApcContext: PVOID,
    IoStatusBlock: PIO_STATUS_BLOCK,
    FsControlCode: u32,
    InputBuffer: PVOID,
    InputBufferLength: u32,
    OutputBuffer: PVOID,
    OutputBufferLength: u32,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x39",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtCreateFile(
    FileHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    IoStatusBlock: PIO_STATUS_BLOCK,
    AllocationSize: PLARGE_INTEGER,
    FileAttributes: u32,
    ShareAccess: u32,
    CreateDisposition: u32,
    CreateOptions: u32,
    EaBuffer: PVOID,
    EaLength: u32,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x55",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtCreateNamedPipeFile(
    FileHandle: PHANDLE,
    DesiredAccess: u32,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    IoStatusBlock: PIO_STATUS_BLOCK,
    ShareAccess: u32,
    CreateDisposition: u32,
    CreateOptions: u32,
    NamedPipeType: u32,
    ReadMode: u32,
    CompletionMode: u32,
    MaximumInstances: u32,
    InboundQuota: u32,
    OutboundQuota: u32,
    DefaultTimeout: PLARGE_INTEGER,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0xBB",
        "syscall",
        "ret"
    );
}


#[unsafe(naked)]
pub extern "system" fn NtOpenFile(
    FileHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    IoStatusBlock: PIO_STATUS_BLOCK,
    ShareAccess: u32,
    OpenOptions: u32,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x33",
        "syscall",
        "ret"
    );
}
  

#[unsafe(naked)]
pub extern "system" fn NtReadFile(
    FileHandle: HANDLE,
    Event: HANDLE,
    ApcRoutine: PIO_APC_ROUTINE,
    ApcContext: PVOID,
    IoStatusBlock: PIO_STATUS_BLOCK,
    Buffer: PVOID,
    Length: u32,
    ByteOffset: PLARGE_INTEGER,
    Key: PULONG,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x6",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtWriteFile(
    FileHandle: HANDLE,
    Event: HANDLE,
    ApcRoutine: PIO_APC_ROUTINE,
    ApcContext: PVOID,
    IoStatusBlock: PIO_STATUS_BLOCK,
    Buffer: PCVOID,
    Length: u32,
    ByteOffset: PLARGE_INTEGER,
    Key: PULONG,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x8",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtDeviceIoControlFile(
    FileHandle: HANDLE,
    Event: HANDLE,
    ApcRoutine: PIO_APC_ROUTINE,
    ApcContext: PVOID,
    IoStatusBlock: PIO_STATUS_BLOCK,
    IoControlCode: u32,
    InputBuffer: PVOID,
    InputBufferLength: u32,
    OutputBuffer: PVOID,
    OutputBufferLength: u32,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x7",
        "syscall",
        "ret"
    );
}


#[unsafe(naked)]
pub extern "system" fn NtDeleteFile(ObjectAttributes: POBJECT_ATTRIBUTES) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0xDB",
        "syscall",
        "ret"
    );
}


#[unsafe(naked)]
pub extern "system" fn NtQueryVirtualMemory(
    ProcessHandle: HANDLE,
    BaseAddress: PVOID,
    MemoryInformationClass: MEMORY_INFORMATION_CLASS,
    MemoryInformation: PVOID,
    MemoryInformationLength: SIZE_T,
    ReturnLength: *mut usize,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x23",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtAllocateVirtualMemory(
    ProcessHandle: HANDLE,
    BaseAddress: *mut PVOID,
    ZeroBits: ULONG_PTR,
    RegionSize: *mut usize,
    AllocationType: u32,
    Protect: u32,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x18",
        "syscall",
        "ret"
    );
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MemExtendedParameter {
    pub type_: u64,
    pub value: u64,
}

#[unsafe(naked)]
pub extern "system" fn NtAllocateVirtualMemoryEx(
    ProcessHandle: HANDLE,
    BaseAddress: *mut PVOID,
    ZeroBits: ULONG_PTR,
    RegionSize: *mut usize,
    AllocationType: u32,
    Protect: u32,
    ExtendedParameters: *mut MemExtendedParameter,
    ExtendedParameterCount: u32
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x78",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtWriteVirtualMemory(
    ProcessHandle: HANDLE,
    BaseAddress: PVOID,
    Buffer: PVOID,
    BufferSize: SIZE_T,
    NumberOfBytesWritten: *mut usize,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x3A",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtReadVirtualMemory(
    ProcessHandle: HANDLE,
    BaseAddress: PVOID,
    Buffer: PVOID,
    BufferSize: SIZE_T,
    NumberOfBytesRead: *mut usize,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x3F",
        "syscall",
        "ret"
    );
}


#[unsafe(naked)]
pub extern "system" fn NtProtectVirtualMemory(
    ProcessHandle: HANDLE,
    BaseAddress: *mut PVOID,
    RegionSize: *mut usize,
    NewProtect: u32,
    OldProtect: *mut u32,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x50",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtTerminateThread(
    ThreadHandle: HANDLE,
    ExitStatus: i32,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x53",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtSuspendProcess(
    ProcessHandle: HANDLE,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x1CE",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtTerminateProcess(
    ProcessHandle: HANDLE,
    ExitStatus: i32,
) -> ! {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x2C",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtDelayExecution(
    Alertable: BOOLEAN,
    DelayInterval: PLARGE_INTEGER,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x34",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtSetInformationThread(
    ThreadHandle: HANDLE,
    ThreadInformationClass: THREADINFOCLASS,
    ThreadInformation: PVOID,
    ThreadInformationLength: u32,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0xD",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtCreateThreadEx(
    ThreadHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    ProcessHandle: HANDLE,
    StartRoutine: PVOID,
    Argument: PVOID,
    CreateFlags: u32,
    ZeroBits: SIZE_T,
    StackSize: SIZE_T,
    MaximumStackSize: SIZE_T,
    AttributeList: PPS_ATTRIBUTE_LIST,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0xC9",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtWaitForSingleObject(
    Handle: HANDLE,
    Alertable: BOOLEAN,
    Timeout: PLARGE_INTEGER,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x4",
        "syscall",
        "ret"
    );
}


#[unsafe(naked)]
pub extern "system" fn NtSuspendThread(
    ThreadHandle: HANDLE,
    PreviousSuspendCount: PULONG,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x1CF",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtFlushBuffersFile(
    FileHandle: HANDLE,
    IoStatusBlock: PIO_STATUS_BLOCK,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x4B",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtFlushBuffersFileEx(
    FileHandle: HANDLE,
    Flags: u32,
    Parameters: PVOID,
    ParametersSize: u32,
    IoStatusBlock: PIO_STATUS_BLOCK,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0xEF",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtQuerySection(
    SectionHandle: HANDLE,
    SectionInformationClass: SECTION_INFORMATION_CLASS,
    SectionInformation: PVOID,
    SectionInformationLength: SIZE_T,
    ReturnLength: *mut usize,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x51",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtCreateSection(
    SectionHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    MaximumSize: PLARGE_INTEGER,
    SectionPageProtection: u32,
    AllocationAttributes: u32,
    FileHandle: HANDLE,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x4A",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtMapViewOfSection(
    SectionHandle: HANDLE,
    ProcessHandle: HANDLE,
    BaseAddress: *mut PVOID,
    ZeroBits: ULONG_PTR,
    CommitSize: SIZE_T,
    SectionOffset: PLARGE_INTEGER,
    ViewSize: *mut usize,
    InheritDisposition: SECTION_INHERIT,
    AllocationType: u32,
    Win32Protect: u32,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x28",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtCreateUserProcess(
    ProcessHandle: PHANDLE,
    ThreadHandle: PHANDLE,
    ProcessDesiredAccess: ACCESS_MASK,
    ThreadDesiredAccess: ACCESS_MASK,
    ProcessObjectAttributes: POBJECT_ATTRIBUTES,
    ThreadObjectAttributes: POBJECT_ATTRIBUTES,
    ProcessFlags: u32,
    ThreadFlags: u32,
    ProcessParameters: PVOID,
    CreateInfo: PPS_CREATE_INFO,
    AttributeList: PPS_ATTRIBUTE_LIST,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0xD1",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtCreateEvent(
    EventHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    EventType: EVENT_TYPE,
    InitialState: BOOLEAN,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x48",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn  NtOpenEvent(
    EventHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x40",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtSetEvent(
    EventHandle: HANDLE,
    PreviousState: PLONG,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0xE",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtSetSystemInformation(
    SystemInformationClass: SYSTEM_INFORMATION_CLASS,
    SystemInformation: PVOID,
    SystemInformationLength: u32,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x1BC",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtSetInformationProcess(
    ProcessHandle: HANDLE,
    ProcessInformationClass: PROCESSINFOCLASS,
    ProcessInformation: PVOID,
    ProcessInformationLength: u32,
) -> NtStatus {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x1C",
        "syscall",
        "ret"
    );
}