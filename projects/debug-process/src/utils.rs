use toolkit::{ProcessMemoryReader, ProcessMemoryWriter, ProcessQuerier};


pub fn is_remote_process_debugged(process_handle: *mut winapi::ctypes::c_void ) -> bool {
    let peb_ptr = ProcessQuerier::query_peb(process_handle).unwrap();
    let being_debugged_address = peb_ptr as usize + 0x2;
    
    let being_debugged = ProcessMemoryReader::read_remote::<u8>(
        process_handle,
        being_debugged_address as *mut _
    ).unwrap();

    being_debugged == 1
}

pub fn set_remote_process_debugged(process_handle: *mut winapi::ctypes::c_void, value: u8) {
    let peb_ptr = ProcessQuerier::query_peb(process_handle).unwrap();
    let being_debugged_address = peb_ptr as usize + 0x2;
    ProcessMemoryWriter::write_remote(process_handle, being_debugged_address as _, &[value]).unwrap();
}