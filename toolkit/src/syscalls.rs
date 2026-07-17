use core::arch::naked_asm;
use ntapi::{ntioapi::{FILE_INFORMATION_CLASS, PIO_APC_ROUTINE, PIO_STATUS_BLOCK}, ntmmapi::MEMORY_INFORMATION_CLASS, ntpsapi::{PPS_ATTRIBUTE_LIST, THREADINFOCLASS}};
use winapi::{shared::{basetsd::{PSIZE_T, SIZE_T, ULONG_PTR}, minwindef::{PULONG, ULONG}, ntdef::{BOOLEAN, HANDLE, NTSTATUS, PHANDLE, PLARGE_INTEGER, POBJECT_ATTRIBUTES, PVOID}}, um::winnt::ACCESS_MASK};

#[unsafe(naked)]
pub extern "system" fn NtClose(
    Handle: HANDLE,
) -> NTSTATUS {
    naked_asm!(
        "mov r10, rcx",
        #[cfg(feature = "win_25h2")]
        "mov eax, 0xF",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtQueryInformationFile(
    FileHandle: HANDLE,
    IoStatusBlock: PIO_STATUS_BLOCK,
    FileInformation: PVOID,
    Length: ULONG,
    FileInformationClass: FILE_INFORMATION_CLASS,
) -> NTSTATUS {
    naked_asm!(
        "mov r10, rcx",
        #[cfg(feature = "win_25h2")]
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
    ThreadInformationLength: ULONG,
    ReturnLength: PULONG,
) -> NTSTATUS {
    naked_asm!(
        "mov r10, rcx",
        #[cfg(feature = "win_25h2")]
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
    Length: ULONG,
    FileInformationClass: FILE_INFORMATION_CLASS,
) -> NTSTATUS {
    naked_asm!(
        "mov r10, rcx",
        #[cfg(feature = "win_25h2")]
        "mov eax, 0x27",
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
    FileAttributes: ULONG,
    ShareAccess: ULONG,
    CreateDisposition: ULONG,
    CreateOptions: ULONG,
    EaBuffer: PVOID,
    EaLength: ULONG,
) -> NTSTATUS {
    naked_asm!(
        "mov r10, rcx",
        #[cfg(feature = "win_25h2")]
        "mov eax, 0x55",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtCreateNamedPipeFile(
    FileHandle: PHANDLE,
    DesiredAccess: ULONG,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    IoStatusBlock: PIO_STATUS_BLOCK,
    ShareAccess: ULONG,
    CreateDisposition: ULONG,
    CreateOptions: ULONG,
    NamedPipeType: ULONG,
    ReadMode: ULONG,
    CompletionMode: ULONG,
    MaximumInstances: ULONG,
    InboundQuota: ULONG,
    OutboundQuota: ULONG,
    DefaultTimeout: PLARGE_INTEGER,
) -> NTSTATUS {
    naked_asm!(
        "mov r10, rcx",
        #[cfg(feature = "win_25h2")]
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
    ShareAccess: ULONG,
    OpenOptions: ULONG,
) -> NTSTATUS {
    naked_asm!(
        "mov r10, rcx",
        #[cfg(feature = "win_25h2")]
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
    Length: ULONG,
    ByteOffset: PLARGE_INTEGER,
    Key: PULONG,
) -> NTSTATUS {
    naked_asm!(
        "mov r10, rcx",
        #[cfg(feature = "win_25h2")]
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
    Buffer: PVOID,
    Length: ULONG,
    ByteOffset: PLARGE_INTEGER,
    Key: PULONG,
) -> NTSTATUS {
    naked_asm!(
        "mov r10, rcx",
        #[cfg(feature = "win_25h2")]
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
    IoControlCode: ULONG,
    InputBuffer: PVOID,
    InputBufferLength: ULONG,
    OutputBuffer: PVOID,
    OutputBufferLength: ULONG,
) -> NTSTATUS {
    naked_asm!(
        "mov r10, rcx",
        #[cfg(feature = "win_25h2")]
        "mov eax, 0x7",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtDeleteFile(ObjectAttributes: POBJECT_ATTRIBUTES) -> NTSTATUS {
    naked_asm!(
        "mov r10, rcx",
        #[cfg(feature = "win_25h2")]
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
    ReturnLength: PSIZE_T,
) -> NTSTATUS {
    naked_asm!(
        "mov r10, rcx",
        #[cfg(feature = "win_25h2")]
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
    RegionSize: PSIZE_T,
    AllocationType: ULONG,
    Protect: ULONG,
) -> NTSTATUS {
    naked_asm!(
        "mov r10, rcx",
        #[cfg(feature = "win_25h2")]
        "mov eax, 0x18",
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
    NumberOfBytesWritten: PSIZE_T,
) -> NTSTATUS {
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
    NumberOfBytesRead: PSIZE_T,
) -> NTSTATUS {
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
    RegionSize: PSIZE_T,
    NewProtect: ULONG,
    OldProtect: PULONG,
) -> NTSTATUS {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x50",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtTerminateProcess(
    ProcessHandle: HANDLE,
    ExitStatus: NTSTATUS,
) -> NTSTATUS {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0x2C",
        "syscall",
        "ret"
    );
}

// #[inline(always)]
#[unsafe(naked)]
pub extern "system"  fn NtDelayExecution(
    Alertable: BOOLEAN,
    DelayInterval: PLARGE_INTEGER,
) -> NTSTATUS {
    naked_asm!(
        "mov r10, rcx",
        #[cfg(feature = "win_25h2")]
        "mov eax, 0x34",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system"  fn NtCreateThreadEx(
    ThreadHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    ProcessHandle: HANDLE,
    StartRoutine: PVOID,
    Argument: PVOID,
    CreateFlags: ULONG,
    ZeroBits: SIZE_T,
    StackSize: SIZE_T,
    MaximumStackSize: SIZE_T,
    AttributeList: PPS_ATTRIBUTE_LIST,
) -> NTSTATUS {
    naked_asm!(
        "mov r10, rcx",
        #[cfg(feature = "win_25h2")]
        "mov eax, 0xC9",
        "syscall",
        "ret"
    );
}
 
#[unsafe(naked)]
pub extern "system"  fn NtWaitForSingleObject(
    Handle: HANDLE,
    Alertable: BOOLEAN,
    Timeout: PLARGE_INTEGER,
) -> NTSTATUS {
    naked_asm!(
        "mov r10, rcx",
        #[cfg(feature = "win_25h2")]
        "mov eax, 0x4",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtSuspendThread(
    ThreadHandle: HANDLE,
    PreviousSuspendCount: PULONG,
) -> NTSTATUS {
    naked_asm!(
        "mov r10, rcx",
        #[cfg(feature = "win_25h2")]
        "mov eax, 0x1CF",
        "syscall",
        "ret"
    );
}