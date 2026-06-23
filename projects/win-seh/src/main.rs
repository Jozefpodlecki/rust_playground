#![no_std]
#![no_main]
#![allow(static_mut_refs, non_snake_case, non_camel_case_types, unused)]
#![windows_subsystem = "console"]
#![feature(arbitrary_self_types_pointers)]
#![feature(sync_unsafe_cell)]
#![feature(naked_functions_rustic_abi)]
#![feature(ptr_alignment_type)]
#![feature(rustc_attrs)]

#[inline(never)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("panic location {:?}", info.location());
    println!("panic message {:?}", info.message().as_str());
    loop {}
}

#[global_allocator]
static ALLOCATOR: emballoc::Allocator<8192> = emballoc::Allocator::new();

#[macro_use]
extern crate alloc;

use core::{cell::SyncUnsafeCell, ffi::c_void};
use ntapi::ntexapi::NtDelayExecution;
use winapi::{um::winnt::*, vc::excpt::{EXCEPTION_DISPOSITION, ExceptionContinueExecution}};

use crate::{code_writer::FaultingCode, dynamic_section::{DynamicSection, ExceptionData, SetupError}, nt_console::*};

mod crt;
mod nt_console;
mod u16_stack_string;
mod dynamic_section;
mod code_buffer;
mod helpers;
mod types;
mod code_writer;

const EXCEPTION_CONTINUE_SEARCH: u32 = 1;
const EXCEPTION_CONTINUE_EXECUTION: u32 = 0;
const EXCEPTION_CXX_EXCEPTION: u32 = 0xE06D7363;

pub static mut ENTERED: SyncUnsafeCell<bool> = SyncUnsafeCell::new(false);

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {
    if let Err(err) = manual_setup() {
        println!("{:?}", err);
    }

    0
}

pub fn manual_setup() -> Result<(), SetupError> {
    unsafe {
        let section = DynamicSection::new()?;
        section.write_code(&FaultingCode::new().privileged_instruction(), custom_handler as _);
        section.register()?;

        let result = section.call();
        println!("result = 0x{:X}", result);

        Ok(())
    }
}

unsafe extern "system" fn custom_handler(
    exception_record_ptr: *mut EXCEPTION_RECORD,
    _establisher_frame: u64,
    context_record_ptr: *mut CONTEXT,
    dispatcher_context_ptr: *mut DISPATCHER_CONTEXT,
) -> EXCEPTION_DISPOSITION {
    unsafe {
        let handle = -1isize as HANDLE;
        let exception_record = &mut *exception_record_ptr;
        let context = &mut *context_record_ptr;
        let dispatcher_context = *dispatcher_context_ptr;

        println!("ImageBase: 0x{:X}", dispatcher_context.ImageBase);

        let rf = &*(dispatcher_context.FunctionEntry as *const RUNTIME_FUNCTION);
            
        println!("BeginAddress         {:08X}", rf.BeginAddress);
        println!("EndAddress           {:08X}", rf.EndAddress);
        println!("UnwindInfoAddress    {:08X}", unsafe {
            *rf.u.UnwindInfoAddress()
        });

        println!("HandlerData {:p}", dispatcher_context.HandlerData);
        let exception_data_ptr = dispatcher_context.HandlerData as *mut ExceptionData;
        let exception_data = &*exception_data_ptr;
        // println!("exception_data {:?}", (*exception_data).data);

        println!("Exception code: 0x{:08X}", exception_record.ExceptionCode);
        println!("Exception flags: 0x{:X}", exception_record.ExceptionFlags);
        println!("Exception address: 0x{:X}", exception_record.ExceptionAddress as u64);
        
        match exception_record.ExceptionCode {
            0xC0000096 => {
                println!("STATUS_PRIVILEGED_INSTRUCTION - Tried to execute Ring 0 instruction");
                context.Rip += 6;
            }
            0xC0000005 => {
                println!("STATUS_ACCESS_VIOLATION - Read/Write to invalid address");
                context.Rip += 3;
            }
            0xC0000094 => {
                println!("STATUS_INTEGER_DIVIDE_BY_ZERO - Division by zero");
                context.Rip += 3;
            }
            0xC0000095 => {
                println!("STATUS_INTEGER_OVERFLOW - Integer overflow");
                context.Rip += 3;
            }
            0xC00000FD => {
                println!("STATUS_STACK_OVERFLOW - Stack overflow");
                context.Rip += 1;
            }
            0xE06D7363 => {
                println!("C++ exception");
                context.Rip += 1;
            }
            0xC000001D => {
                println!("STATUS_ILLEGAL_INSTRUCTION - UD2 instruction / Invalid opcode");
                context.Rip += 2;
            }
            0x80000003 => {
                println!("STATUS_BREAKPOINT - INT3 instruction");
                context.Rip += 1;
            }
            _ => {
                println!("Unknown exception code");
                context.Rip += 1;
            }
        }
                
        context.Rip += 3;
        
        ExceptionContinueExecution
    }
}