use core::sync::atomic::Ordering;
use super::{Context, ExecutionResult, OpCode};

pub type ContextHandler = extern "C" fn(*mut ExecutionResult, &mut Context) -> ();

pub extern "C" fn handler_add(result: *mut ExecutionResult, context: &mut Context) {
    let a = context.registers[0];
    let b = context.registers[1];
    let res = a.wrapping_add(b);
    
    context.registers[2] = res;
    context.flags = if res == 0 { 1 } else { 0 };
    context.cycles += 10;
    context.execution_count.fetch_add(1, Ordering::Relaxed);
    
    context.op_history[context.history_index as usize % 16] = OpCode::Add;
    context.history_index += 1;
    
    unsafe {
        (*result).return_value = res;
        (*result).exit_code = 0;
        (*result).cycles_used = 10;
        (*result).memory_accessed = 0;
        (*result).opcode_executed = 0;
        (*result).success = true;
        (*result).error_msg = core::ptr::null();
    }
}

pub extern "C" fn handler_sub(result: *mut ExecutionResult, context: &mut Context) {
    let a = context.registers[0];
    let b = context.registers[1];
    let res = a.wrapping_sub(b);
    
    context.registers[2] = res;
    context.flags = if res == 0 { 1 } else { 0 };
    context.cycles += 10;
    context.execution_count.fetch_add(1, Ordering::Relaxed);
    
    context.op_history[context.history_index as usize % 16] = OpCode::Sub;
    context.history_index += 1;
    
    unsafe {
        (*result).return_value = res;
        (*result).exit_code = 0;
        (*result).cycles_used = 10;
        (*result).memory_accessed = 0;
        (*result).opcode_executed = 1;
        (*result).success = true;
        (*result).error_msg = core::ptr::null();
    }
}

pub extern "C" fn handler_mul(result: *mut ExecutionResult, context: &mut Context) {
    let a = context.registers[0];
    let b = context.registers[1];
    let res = a.wrapping_mul(b);
    
    context.registers[2] = res;
    context.flags = if res == 0 { 1 } else { 0 };
    context.cycles += 20;
    context.execution_count.fetch_add(1, Ordering::Relaxed);
    
    context.op_history[context.history_index as usize % 16] = OpCode::Mul;
    context.history_index += 1;
    
    unsafe {
        (*result).return_value = res;
        (*result).exit_code = 0;
        (*result).cycles_used = 20;
        (*result).memory_accessed = 0;
        (*result).opcode_executed = 2;
        (*result).success = true;
        (*result).error_msg = core::ptr::null();
    }
}

pub extern "C" fn handler_div(result: *mut ExecutionResult, context: &mut Context) {
    let a = context.registers[0];
    let b = context.registers[1];
    
    if b == 0 {
        static ERROR_MSG: [u8; 17] = *b"Division by zero\0";
        unsafe {
            (*result).return_value = 0;
            (*result).exit_code = 1;
            (*result).cycles_used = 5;
            (*result).memory_accessed = 0;
            (*result).opcode_executed = 3;
            (*result).success = false;
            (*result).error_msg = ERROR_MSG.as_ptr();
        }
        return;
    }
    
    let res = a / b;
    let remainder = a % b;
    
    context.registers[2] = res;
    context.registers[3] = remainder;
    context.flags = if remainder == 0 { 2 } else { 0 };
    context.cycles += 30;
    context.execution_count.fetch_add(1, Ordering::Relaxed);
    
    context.op_history[context.history_index as usize % 16] = OpCode::Div;
    context.history_index += 1;
    
    unsafe {
        (*result).return_value = res;
        (*result).exit_code = 0;
        (*result).cycles_used = 30;
        (*result).memory_accessed = 0;
        (*result).opcode_executed = 3;
        (*result).success = true;
        (*result).error_msg = core::ptr::null();
    }
}

pub extern "C" fn handler_memory(result: *mut ExecutionResult, context: &mut Context) {
    let addr = context.registers[0] as usize;
    let value = context.registers[1];
    
    if addr < 32 {
        context.data_section[addr] = value;
        context.cache_hits += 1;
    } else {
        context.cache_misses += 1;
        context.cycles += 100;
        static ERROR_MSG: [u8; 28] = *b"Memory access out of bounds\0";
        unsafe {
            (*result).return_value = 0;
            (*result).exit_code = 2;
            (*result).cycles_used = 100;
            (*result).memory_accessed = 0;
            (*result).opcode_executed = 4;
            (*result).success = false;
            (*result).error_msg = ERROR_MSG.as_ptr();
        }
        return;
    }
    
    context.cycles += 5;
    context.execution_count.fetch_add(1, Ordering::Relaxed);
    
    context.op_history[context.history_index as usize % 16] = OpCode::Custom(0xDEADBEEF);
    context.history_index += 1;
    
    unsafe {
        (*result).return_value = value;
        (*result).exit_code = 0;
        (*result).cycles_used = 5;
        (*result).memory_accessed = 8;
        (*result).opcode_executed = 4;
        (*result).success = true;
        (*result).error_msg = core::ptr::null();
    }
}