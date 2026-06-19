#![allow(unconditional_panic)]

use ntapi::ntrtl::*;
use winapi::{um::{errhandlingapi::AddVectoredExceptionHandler, minwinbase::{EXCEPTION_ILLEGAL_INSTRUCTION, EXCEPTION_INT_DIVIDE_BY_ZERO}, winnt::EXCEPTION_POINTERS}, vc::excpt::*};

use crate::{api::*, exceptions::*, manual::rtl_add_veh, types::*};

mod types;
mod exceptions;
mod memory;
mod api;
mod manual;

unsafe extern "system" fn handle_invalid_opcode(
    exception_info: *mut EXCEPTION_POINTERS,
) -> i32 {
    unsafe {
        let info = *exception_info;
        let record = *info.ExceptionRecord;
        
        if record.ExceptionCode == EXCEPTION_ILLEGAL_INSTRUCTION {
            println!("Handled invalid opcode (UD2)!");
            
            // UD2 is 2 bytes
            (*info.ContextRecord).Rip += 2;
            
            return EXCEPTION_CONTINUE_EXECUTION;
        }
    }

    println!("Unhandled continuing search");
    EXCEPTION_CONTINUE_SEARCH 
}

unsafe extern "system" fn handle_div_by_zero(
    exception_info: *mut EXCEPTION_POINTERS,
) -> i32 {

    unsafe {
        let info = *exception_info;
        let record = *info.ExceptionRecord;
        
        if record.ExceptionCode == EXCEPTION_INT_DIVIDE_BY_ZERO {
            println!("Handled divide by zero!");
            
            // idiv
            (*info.ContextRecord).Rip += 2;
            
            return EXCEPTION_CONTINUE_EXECUTION;
        }
    }

    println!("Unhandled continuing search");
    EXCEPTION_CONTINUE_SEARCH 
}

unsafe extern "system" fn print_and_continue(
    exception_info: *mut EXCEPTION_POINTERS,
) -> i32 {

    unsafe {
        let info = *exception_info;
        
        println!("\nEXCEPTION RECORD:");
        let rec = VehExceptionRecord(*info.ExceptionRecord);
        println!("{:#?}", rec);
        
        println!("\nCONTEXT (Registers at time of exception):");
        let ctx = VehContext(*info.ContextRecord);
        println!("{:#?}", ctx);
        
        println!("\nContext Flags Decoded:");
        println!("   {}", ctx.context_flags_string());
    }
    
    println!("\n➜ Returning EXCEPTION_CONTINUE_SEARCH (will crash)");
    EXCEPTION_CONTINUE_SEARCH
}

pub fn print_veh_entries() {
    unsafe {
        let ntdll_base: usize = get_ntdll_base() as _;
        let list_ptr = (ntdll_base + 0x1E9578) as *mut VECTORED_HANDLER_LIST;
        let list = *list_ptr;
        
        println!("=== VEH Handler List ===");
        println!("First: {:p}, Last: {:p}\n", list.first_exception_handler, list.last_exception_handler);
        
        let sentinel = (list_ptr as *mut u8).add(8) as *mut LIST_ENTRY;
        let mut current = list.first_exception_handler;
        let mut index = 0;
        
        while !current.is_null() && current as *mut LIST_ENTRY != sentinel {
            let entry = *current;
            let decoded = RtlDecodePointer(entry.handler as _);
            
            println!("[{}] Entry: {:p}", index, current);
            println!("    flink: {:p}, blink: {:p}", entry.entry.flink, entry.entry.blink);
            println!("    sync_refs: {:p}", entry.sync_refs);
            println!("    padding: 0x{:X}, rnd_upper: 0x{:X}", entry.padding, entry.rnd_upper);
            println!("    handler (encoded): {:p}", entry.handler);
            println!("    handler (decoded): {:p}\n", decoded);
            
            current = entry.entry.flink as *mut VEH_HANDLER_ENTRY;
            index += 1;
        }
        
        println!("=== End of list ({} entries) ===", index);
    }
}

fn main() {
    unsafe {
        AddVectoredExceptionHandler(0, Some(handle_invalid_opcode));
        RtlAddVectoredExceptionHandler(0, Some(handle_div_by_zero));
        
        let veh_handler_addr = print_and_continue as *const ();
        let process_handle: *mut winapi::ctypes::c_void = -1 as _;
        rtl_add_veh(process_handle, veh_handler_addr as _);

        print_veh_entries();

        println!("triggering do_invalid_opcode");

        do_invalid_opcode();

        println!("next do_divide_by_zero");

        do_divide_by_zero();
        
        println!("next do_access_violation");

        do_access_violation();
    }
}