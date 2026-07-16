use core::arch::naked_asm;
use super::{Context, ExecutionResult, OpCode};

#[unsafe(naked)]
pub unsafe extern "C" fn jump_dispatch(opcode: u64, context: &mut Context, result: *mut ExecutionResult) {
    naked_asm!(
        "cmp rcx, 5",
        "jae 1f",
        "lea rax, [rip + HANDLER_TABLE]",
        "mov rbx, rcx",
        "imul rbx, 8",
        "add rax, rbx",
        "mov rax, [rax]",
        "mov rcx, r8",      // result pointer (3rd arg)
        "mov rdx, rdx",     // context pointer (2nd arg)
        "jmp rax",
        "1:",
        "mov rax, 0",
        "mov rdx, 1",
        "xor r8, r8",
        "xor r9, r9",
        "ret",
    );
}

pub fn execute_opcode(opcode: OpCode, context: &mut Context) -> ExecutionResult {
    let mut result = ExecutionResult {
        return_value: 0,
        exit_code: 0,
        cycles_used: 0,
        memory_accessed: 0,
        opcode_executed: 0,
        success: false,
        error_msg: core::ptr::null(),
    };
    
    let opcode_value = match opcode {
        OpCode::Add => 0,
        OpCode::Sub => 1,
        OpCode::Mul => 2,
        OpCode::Div => 3,
        OpCode::Custom(_) => 4,
    };
    
    unsafe { jump_dispatch(opcode_value, context, &mut result) };
    result
}