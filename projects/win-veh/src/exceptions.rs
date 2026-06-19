use core::arch::naked_asm;

#[cfg(target_arch = "x86_64")]
#[unsafe(naked)]
pub unsafe extern "C" fn do_invalid_opcode() {
    naked_asm!(
        "ud2",              // Invalid opcode - triggers EXCEPTION_ILLEGAL_INSTRUCTION
        "ret",
    );
}

#[cfg(target_arch = "x86_64")]
#[unsafe(naked)]
pub unsafe extern "C" fn do_divide_by_zero() {
    naked_asm!(
        "mov rax, 42",
        "xor rdx, rdx",
        "mov rcx, 0",
        "idiv rcx",
        "ret",
    );
}

#[cfg(target_arch = "x86_64")]
#[unsafe(naked)]
pub unsafe extern "C" fn do_access_violation() -> i32 {
    naked_asm!(
        "xor rax, rax",     // RAX = 0 (null pointer)
        "mov rax, [rax]",   // Read from address 0 - ACCESS VIOLATION!
        "ret",              // Return the value (never reached)
    );
}

#[cfg(target_arch = "x86_64")]
#[unsafe(naked)]
pub unsafe extern "C" fn do_access_violation_address() -> i32 {
    naked_asm!(
        "mov rax, 0x12345678",    // Some invalid address
        "mov rax, [rax]",         // Read from it - ACCESS VIOLATION!
        "ret",
    );
}

#[unsafe(naked)]
pub unsafe extern "C" fn write_violation() {
    naked_asm!(
        "mov rax, 0x12345678",
        "mov [rax], rcx",   // ← WRITE to 0x12345678 - ExceptionInformation[0] = 1
        "ret",
    );
}

#[unsafe(naked)]
pub unsafe extern "C" fn execute_violation() {
    naked_asm!(
        "mov rax, 0x12345678",
        "jmp rax",          // ← EXECUTE at 0x12345678 - ExceptionInformation[0] = 8
        "ret",
    );
}