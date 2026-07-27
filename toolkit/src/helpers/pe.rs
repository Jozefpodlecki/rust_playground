
use ntapi::{ntpebteb::PEB, ntpsapi::NtCurrentProcess};
use winapi::{ctypes::c_void, shared::ntdef::NTSTATUS, um::winnt::{IMAGE_DOS_HEADER, IMAGE_NT_HEADERS64}};

use crate::ProcessMemoryReader;

pub struct PEHeader;

impl PEHeader {
    pub fn entrypoint(peb_ptr: *const c_void) -> Result<*mut c_void, NTSTATUS> {
        let handle = NtCurrentProcess;
        let peb: PEB = ProcessMemoryReader::read_remote(handle, peb_ptr as _)?;
        let image_base = peb.ImageBaseAddress;

        let dos_header: IMAGE_DOS_HEADER = 
            ProcessMemoryReader::read_remote(handle, image_base)?;

        let nt_headers_addr = (image_base as usize + dos_header.e_lfanew as usize) as *const c_void;
        let nt_headers: IMAGE_NT_HEADERS64 = 
            ProcessMemoryReader::read_remote(handle, nt_headers_addr as _)?;

        let entrypoint_rva = nt_headers.OptionalHeader.AddressOfEntryPoint;
        Ok((image_base as usize + entrypoint_rva as usize) as *mut c_void)
    }
}