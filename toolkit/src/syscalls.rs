use core::arch::naked_asm;
use ntapi::ntioapi::{PIO_APC_ROUTINE, PIO_STATUS_BLOCK};
use winapi::shared::{basetsd::{PSIZE_T, SIZE_T}, minwindef::{PULONG, ULONG}, ntdef::{HANDLE, NTSTATUS, PLARGE_INTEGER, POBJECT_ATTRIBUTES, PVOID}};

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
        "mov eax, 0x7",
        "syscall",
        "ret"
    );
}

#[unsafe(naked)]
pub extern "system" fn NtDeleteFile(ObjectAttributes: POBJECT_ATTRIBUTES) -> NTSTATUS {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0xDB",
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