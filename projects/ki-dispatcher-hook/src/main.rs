#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![allow(internal_features)]
#![feature(core_intrinsics)]

use core::{arch::naked_asm, panic::PanicInfo};

use iced_x86::{Decoder, DecoderOptions};
use ntapi::{ntpsapi::NtCurrentProcess, ntxcapi::NtContinue};
use toolkit::{NtDll, ProcessMemoryReader, Sleeper, println, syscalls::NtTerminateProcess};
use winapi::{um::winnt::EXCEPTION_POINTERS, vc::excpt::*};

use crate::exceptions::*;

extern crate builtins;

mod exceptions;

#[macro_use]
extern crate alloc;

#[global_allocator]
static ALLOCATOR: emballoc::Allocator<10485760> = emballoc::Allocator::new();

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{:?}", info);
    loop {}
}

unsafe extern "system" fn handle_exception(
    exception_info: *mut EXCEPTION_POINTERS,
) -> i32 {
    let exception = unsafe { &*exception_info };
    let record = unsafe { &*exception.ExceptionRecord };
    println!("ExceptionRecord.ExceptionCode 0x{:X}", record.ExceptionCode);
    println!("ExceptionRecord.ExceptionAddress {:p}", record.ExceptionAddress);
    println!("ExceptionRecord.ExceptionFlags 0x{:X}", record.ExceptionFlags);
    let context = unsafe { &*exception.ContextRecord };
    
    println!("Context.Rip 0x{:X}", context.Rip);
    // EXCEPTION_INT_DIVIDE_BY_ZERO
    EXCEPTION_CONTINUE_SEARCH 
}

#[allow(non_snake_case)]
#[inline(never)]
#[unsafe(no_mangle)]
pub extern "system" fn KiUserExceptionDispatcherHandler(
    exception_record: *mut winapi::um::winnt::EXCEPTION_RECORD,
    context: *mut winapi::um::winnt::CONTEXT,
) -> ! {
    unsafe {
        println!("ExceptionRecord: {:p}", exception_record);
        println!("Context: {:p}", context);
        
        if let Some(record) = exception_record.as_ref() {
            println!("ExceptionCode: 0x{:X}", record.ExceptionCode);
            println!("ExceptionAddress: {:p}", record.ExceptionAddress);
        }

        // let bytes = ProcessMemoryReader::read_bytes_fixed::<64>(exception_record as _).unwrap();
        // println!("{}", bytes);

        if let Some(ctx) = context.as_mut() {
            let bytes = ProcessMemoryReader::read_bytes_fixed::<16>(ctx.Rip as _).unwrap();
            let mut decoder = Decoder::new(64, bytes.as_ref(), DecoderOptions::NONE);

            let instr = decoder.decode();
            println!("{:?} {}", instr.code(), instr.len());

            if instr.is_invalid() {
                let error = decoder.last_error();
                println!("{:?}", error);
                NtTerminateProcess(NtCurrentProcess, 0);
            }

            ctx.Rip += instr.len() as u64;

            let status = NtContinue(context, 0);
            if status != 0 {
                println!("NtContinue failed: 0x{:X}", status);
                NtTerminateProcess(NtCurrentProcess, 0);
            }
        }

        core::intrinsics::unreachable()   
    }
}

#[unsafe(naked)]
unsafe extern "system" fn KiUserExceptionDispatcherHook() {
    naked_asm!(

        "cld",
        "mov rcx, rsp",
        "add rcx, 0x4F0",
        "mov rdx, rsp",
        "call KiUserExceptionDispatcherHandler",
    );
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {

    unsafe {
        let ntdll = NtDll::from_current_process();
        let KiUserExceptionDispatcher = ntdll.base() as usize + 0x1640C0;
        let hook_addr=  KiUserExceptionDispatcherHook as *const () as usize;
        let buffer = toolkit::hook::jmp_trampoline_to::<13>(hook_addr as _).unwrap();
        toolkit::hook::hook_function(
            NtCurrentProcess,
            KiUserExceptionDispatcher as *const () as usize,
            hook_addr, buffer.as_slice()).unwrap();
        
        // RtlAddVectoredExceptionHandler(0, Some(handle_exception));
            
        do_int3();
        do_privileged_instruction();
        do_invalid_opcode();
        do_divide_by_zero();
        println!("do_divide_by_zero");
    }

    0
}
