use core::{mem::zeroed, ptr::null_mut};
use core::mem;

use ntapi::{ntapi_base::CLIENT_ID, ntmmapi::{NtFreeVirtualMemory, NtQuerySection, SECTION_BASIC_INFORMATION, SectionBasicInformation}, ntpebteb::PEB, ntpsapi::NtOpenProcess, ntrtl::RTL_USER_PROCESS_PARAMETERS};
use winapi::shared::ntdef::NTSTATUS;
use winapi::{shared::{ntdef::{HANDLE, NT_SUCCESS, OBJECT_ATTRIBUTES, PVOID}}, um::winnt::PROCESS_ALL_ACCESS};

use crate::syscalls::{NtAllocateVirtualMemory, NtClose, NtReadVirtualMemory};

#[derive(Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub tid: u32,
    pub entry_point: PVOID,
    pub peb_addr: u64,
    pub section_handle: PVOID,
    pub file_handle: PVOID,
    pub params_addr: u64,
    pub output_flags: u32,
    pub manifest_addr: u64,
    pub manifest_size: u32,
}

impl ProcessInfo {
    fn open_process(&self, pid: u32) -> Result<HANDLE, NTSTATUS> {
        let mut handle: HANDLE = null_mut();
        let mut obj_attr: OBJECT_ATTRIBUTES = unsafe { zeroed() };
        let mut parent_client: CLIENT_ID = unsafe { zeroed() };
        parent_client.UniqueProcess = pid as _;
        
        let status = unsafe {
            NtOpenProcess(
                &mut handle as *mut _ as _,
                PROCESS_ALL_ACCESS,
                &mut obj_attr,
                &mut parent_client
            )
        };
        
        if status < 0 {
            return Err(status);
        }
        
        Ok(handle)
    }

    pub fn peb(&self) -> Result<PEB, NTSTATUS> {
        let handle = self.open_process(self.pid)?;

        let peb: PEB = read_remote_bytes(handle, self.peb_addr as _)?;

        Ok(peb)
    }

    pub fn process_parameters(&self) -> Result<RTL_USER_PROCESS_PARAMETERS, NTSTATUS> {
        let handle = self.open_process(self.pid)?;
        let peb: PEB = read_remote_bytes(handle, self.peb_addr as _)?;
        let process_params: RTL_USER_PROCESS_PARAMETERS = read_remote_bytes(handle, peb.ProcessParameters as _)?;
        NtClose(handle);

        Ok(process_params)
    }

    pub fn command_args(&self) -> Result<RemoteUnicodeString, NTSTATUS> {
        let handle = self.open_process(self.pid)?;
        let peb: PEB = read_remote_bytes(handle, self.peb_addr as _)?;
        let process_params: RTL_USER_PROCESS_PARAMETERS = read_remote_bytes(handle, peb.ProcessParameters as _)?;
        
        let cmd_line = process_params.CommandLine;
        let size = cmd_line.Length as usize;
        let buffer = cmd_line.Buffer;
        
        let mut local_addr = null_mut();
        let mut alloc_size = size;
        let status = unsafe {
            NtAllocateVirtualMemory(
                handle,
                &mut local_addr,
                0,
                &mut alloc_size,
                0x3000,
                0x04,
            )
        };
        
        if !NT_SUCCESS(status) {
            NtClose(handle);
            return Err(status);
        }
        
        let mut bytes_read = 0;
        let status = unsafe {
            NtReadVirtualMemory(
                handle,
                buffer as _,
                local_addr,
                size,
                &mut bytes_read,
            )
        };
        
        NtClose(handle);
        
        if !NT_SUCCESS(status) {
            let _ = unsafe { NtFreeVirtualMemory(handle, &mut local_addr, &mut alloc_size, 0x8000) };
            return Err(status);
        }
        
        Ok(RemoteUnicodeString {
            ptr: local_addr,
            size: bytes_read,
        })
    }

    pub fn get_basic_section(&self) -> Result<SECTION_BASIC_INFORMATION, NTSTATUS> {
        let mut basic_info: SECTION_BASIC_INFORMATION = unsafe { zeroed() };
        let mut return_length: usize = 0;
        
        let status = unsafe {
            NtQuerySection(
                self.section_handle,
                SectionBasicInformation,
                &mut basic_info as *mut _ as _,
                mem::size_of::<SECTION_BASIC_INFORMATION>(),
                &mut return_length,
            )
        };
        
        if !NT_SUCCESS(status) {
            return Err(status);
        }
        
        Ok(basic_info)
    }

    pub fn environment(&self) -> Result<RemoteEnvironment, NTSTATUS> {
        let handle = self.open_process(self.pid)?;
        let peb: PEB = read_remote_bytes(handle, self.peb_addr as _)?;
        let process_params: RTL_USER_PROCESS_PARAMETERS = read_remote_bytes(handle, peb.ProcessParameters as _)?;

        let mut env_size = process_params.EnvironmentSize as usize;
        let mut local_addr = null_mut();
        let mut size = env_size;

        let status = unsafe {
            NtAllocateVirtualMemory(
                handle,
                &mut local_addr,
                0,
                &mut size,
                0x3000,
                0x04,
            )
        };

        if !NT_SUCCESS(status) {
            return Err(status);
        }

        let mut bytes_read = 0;
        let status = unsafe {
            NtReadVirtualMemory(
                handle,
                process_params.Environment as _,
                local_addr,
                env_size,
                &mut bytes_read,
            )
        };

        if !NT_SUCCESS(status) {
            let _ = unsafe { NtFreeVirtualMemory(handle, &mut local_addr, &mut env_size, 0x8000) };
            return Err(status);
        }

        NtClose(handle);

        Ok(RemoteEnvironment {
            ptr: local_addr,
            size: bytes_read,
        })
    }
}

pub fn read_remote_bytes_arr<const N: usize>(handle: HANDLE, address: PVOID) -> Result<[u8; N], NTSTATUS> {
    let mut bytes_read: usize = 0;
    let mut buffer = [0u8; N];
    
    let status = unsafe {
        NtReadVirtualMemory(
            handle,
            address,
            buffer.as_mut_ptr() as _,
            N,
            &mut bytes_read,
        )
    };

    if NT_SUCCESS(status) && bytes_read == N {
        Ok(buffer)
    } else {
        Err(status)
    }
}

pub fn read_remote_bytes<T>(handle: HANDLE, address: PVOID) -> Result<T, NTSTATUS> {
    let mut bytes_read: usize = 0;
    let mut buffer: T = unsafe { zeroed() };
    let size = core::mem::size_of::<T>();
    
    let status = unsafe {
        crate::syscalls::NtReadVirtualMemory(
            handle,
            address,
            &mut buffer as *mut _ as _,
            size,
            &mut bytes_read,
        )
    };

    if NT_SUCCESS(status) && bytes_read == size {
        Ok(buffer)
    } else {
        Err(status)
    }
}

pub struct RemoteUnicodeString {
    ptr: PVOID,
    size: usize,
}

impl RemoteUnicodeString {
    pub fn as_ptr(&self) -> PVOID {
        self.ptr
    }

    pub fn as_u16_slice(&self) -> &[u16] {
        unsafe { core::slice::from_raw_parts(self.ptr as *const u16, self.size / 2) }
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl Drop for RemoteUnicodeString {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
        }
    }
}

pub struct RemoteEnvironment {
    ptr: PVOID,
    size: usize,
}

impl RemoteEnvironment {
    pub fn as_ptr(&self) -> PVOID {
        self.ptr
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.ptr as *const u8, self.size) }
    }

    pub fn as_u16_slice(&self) -> &[u16] {
        unsafe { core::slice::from_raw_parts(self.ptr as *const u16, self.size / 2) }
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl Drop for RemoteEnvironment {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            
        }
    }
}