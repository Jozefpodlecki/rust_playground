use core::arch::naked_asm;


#[unsafe(naked)]
pub unsafe extern "C" fn do_int3() {
    naked_asm!(
        "int3",
        "ret",
    );
}

#[unsafe(naked)]
pub unsafe extern "C" fn do_privileged_instruction() {
    naked_asm!(
        "mov rax, cr0",
        "ret",
    );
}

#[unsafe(naked)]
pub unsafe extern "C" fn do_invalid_opcode() {
    naked_asm!(
        "ud2",
        "ret",
    );
}

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