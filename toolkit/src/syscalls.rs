use core::arch::naked_asm;
use winapi::shared::ntdef::{NTSTATUS, POBJECT_ATTRIBUTES};

#[unsafe(naked)]
pub fn NtDeleteFile(ObjectAttributes: POBJECT_ATTRIBUTES) -> NTSTATUS {
    naked_asm!(
        "mov r10, rcx",
        "mov eax, 0xDB",
        "syscall",
        "ret"
    );
}