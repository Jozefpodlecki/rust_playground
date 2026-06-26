use ntapi::ntrtl::*;
use utils::NtDll;
use winapi::{shared::ntdef::HANDLE, um::winnt::{PAGE_EXECUTE_READWRITE, PAGE_READWRITE}};

use crate::{api::*, types::*};

pub fn rtl_add_veh(process_handle: HANDLE, veh_handler_addr: *mut winapi::ctypes::c_void) {
    unsafe {
        let encoded_handler = RtlEncodePointer(veh_handler_addr as _);

        let ntdll = NtDll::from_current_process();
        let list_ptr = ntdll.vectored_handler_list() as *mut VECTORED_HANDLER_LIST;
        let mut list = *list_ptr;
        
        let entry_addr = alloc_memory_at_address(
            process_handle,
            None,
            core::mem::size_of::<VEH_HANDLER_ENTRY>(),
            PAGE_READWRITE
        ).unwrap();
        
        let sync_addr = alloc_memory_at_address(
            process_handle,
            None,
            core::mem::size_of::<usize>(),
            PAGE_READWRITE
        ).unwrap();
        
        let entry = VEH_HANDLER_ENTRY {
            entry: LIST_ENTRY {
                flink: (list_ptr as *mut u8).add(8) as *mut LIST_ENTRY,
                blink: list.last_exception_handler as _,
            },
            sync_refs: sync_addr as *mut _,
            padding: 0,
            rnd_upper: 1234,
            handler: encoded_handler as _,
        };

        write_value_to_address(process_handle, sync_addr, &1_usize).unwrap();
        write_value_to_address(process_handle, entry_addr, &entry).unwrap();

        let old_last = list.last_exception_handler;
        if !old_last.is_null() {
            let mut old_last_entry = *old_last;
            old_last_entry.entry.flink = entry_addr as *mut LIST_ENTRY;
            write_value_to_address(process_handle, old_last as usize, &old_last_entry).unwrap();
        }

        list.last_exception_handler = entry_addr as *mut VEH_HANDLER_ENTRY;

        let list_size = core::mem::size_of::<VECTORED_HANDLER_LIST>();
        let old_protect = protect_memory_at_address(
            process_handle,
            list_ptr as usize,
            list_size,
            PAGE_READWRITE
        ).unwrap();

        write_value_to_address(process_handle, list_ptr as usize, &list).unwrap();

        protect_memory_at_address(
            process_handle,
            list_ptr as usize,
            list_size,
            old_protect
        ).unwrap();

        protect_memory_at_address(
            process_handle,
            entry_addr,
            core::mem::size_of::<VEH_HANDLER_ENTRY>(),
            PAGE_EXECUTE_READWRITE
        ).unwrap();
    }
}
