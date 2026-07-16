#![no_std]
#![no_main]
#![windows_subsystem = "console"]

use core::panic::PanicInfo;
use toolkit::println;

extern crate builtins;

mod context;
mod handlers;
mod dispatch;

use context::Context;
use dispatch::execute_opcode;
use context::ExecutionResult;
use context::OpCode;

use crate::handlers::ContextHandler;
use crate::handlers::*;

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub static HANDLER_TABLE: [ContextHandler; 5] = [
    handler_add,
    handler_sub,
    handler_mul,
    handler_div,
    handler_memory,
];

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {
    println!("=== Advanced Jump Table System ===");
    println!();
    
    let mut context = Context {
        registers: [10, 5, 0, 0, 0, 0, 0, 0],
        flags: 0,
        pc: 0,
        stack_ptr: 0x1000,
        op_history: [OpCode::Add; 16],
        history_index: 0,
        execution_count: core::sync::atomic::AtomicU64::new(0),
        data_section: [0; 32],
        heap_ptr: 0x2000,
        heap_size: 4096,
        cycles: 0,
        cache_hits: 0,
        cache_misses: 0,
    };
    
    println!("Test 1: Sequential execution of all handlers");
    println!("-------------------------------------------");
    
    let opcodes = [OpCode::Add, OpCode::Sub, OpCode::Mul, OpCode::Div, OpCode::Custom(0)];
    for opcode in opcodes.iter() {
        context.registers[0] = 100;
        context.registers[1] = 25;
        context.registers[2] = 0;
        
        println!("Opcode {:?}:", opcode);
        println!("  Before: R0={}, R1={}, R2={}", 
            context.registers[0], context.registers[1], context.registers[2]);
        
        let result = execute_opcode(*opcode, &mut context);
        
        println!("  After:  R0={}, R1={}, R2={}", 
            context.registers[0], context.registers[1], context.registers[2]);
        println!("  Result: return={}, exit={}, cycles={}, memory={}, success={}", 
            result.return_value, result.exit_code, result.cycles_used, result.memory_accessed, result.success);
        
        if !result.success && !result.error_msg.is_null() {
            let msg = unsafe { core::ffi::CStr::from_ptr(result.error_msg as *const core::ffi::c_char) };
            println!("  Error: {:?}", msg);
        }
        println!("  Total cycles: {}", context.cycles);
        println!();
    }
    
    println!();
    println!("All tests completed successfully!");
    0
}