
use core::{mem, ops::{Deref, DerefMut}, ptr, sync::atomic::{AtomicBool, Ordering}};

use ntapi::{ntapi_base::CLIENT_ID, ntexapi::*, ntobapi::NtClose, ntpebteb::PPEB, ntpsapi::{NtOpenProcess, NtQueryInformationProcess, NtTerminateProcess, PROCESS_BASIC_INFORMATION, ProcessBasicInformation, ProcessImageFileName, ProcessImageFileNameWin32}};
use crate::{U8CStackString, U16CStackString, println};
use winapi::{shared::{minwindef::FALSE, ntdef::{HANDLE, NTSTATUS, OBJECT_ATTRIBUTES, UNICODE_STRING}, ntstatus::{STATUS_INFO_LENGTH_MISMATCH, STATUS_SUCCESS}}, um::{errhandlingapi::GetLastError, handleapi::CloseHandle, processthreadsapi::{CreateProcessW, PROCESS_INFORMATION, STARTUPINFOW, TerminateProcess}, winbase::{CREATE_SUSPENDED, DETACHED_PROCESS}, winnt::{PROCESS_ALL_ACCESS, PROCESS_QUERY_LIMITED_INFORMATION}} };

const BUFFER_SIZE: usize = 2_000_000;
static mut BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

pub struct SystemProcessIterator {
    index: usize,
    count: usize,
}

impl SystemProcessIterator {
    
    pub fn new() -> Result<Self, NTSTATUS> {
        unsafe {
            const STATUS_SUCCESS: i32 = 0;
            let mut return_length = 0;

            let status = NtQuerySystemInformation(
                SystemProcessInformation,
                BUFFER.as_mut_ptr() as *mut _,
                BUFFER_SIZE as u32,
                &mut return_length,
            );

            let mut count = 0;
            let mut current_ptr = BUFFER.as_ptr();
            
            loop {
                let info = current_ptr as *const SYSTEM_PROCESS_INFORMATION;
                count += 1;
                
                let offset = (*info).NextEntryOffset;
                if offset == 0 {
                    break;
                }
                current_ptr = current_ptr.add(offset as usize);
            }

            Ok(SystemProcessIterator {
                index: 0,
                count,
            })
        }
    }

    pub fn total_count(&self) -> usize {
        self.count
    }
}


#[derive(Clone, Copy)]
pub struct SystemProcessInfo(pub (crate) SYSTEM_PROCESS_INFORMATION);

impl SystemProcessInfo {
    pub fn pid(&self) -> u32 {
        self.0.UniqueProcessId as u32
    }

    pub fn parent_pid(&self) -> u32 {
        self.0.InheritedFromUniqueProcessId as u32
    }

    pub fn thread_count(&self) -> u32 {
        self.0.NumberOfThreads
    }

    pub fn handle_count(&self) -> u32 {
        self.0.HandleCount
    }

    pub fn name(&self) -> U16CStackString<128> {
        unsafe {
            if self.0.ImageName.Length > 0 && !self.0.ImageName.Buffer.is_null() {
                let slice = core::slice::from_raw_parts(
                    self.0.ImageName.Buffer,
                    self.0.ImageName.Length as usize / 2,
                );
                let mut result = U16CStackString::<128>::new();
                for &ch in slice {
                    result.push(ch);
                }
                result
            } else {
                U16CStackString::new()
            }
        }
    }

     pub fn path_via_spiinf(&self) -> U16CStackString<260> {
        let pid = self.pid();
        
        unsafe {
            let mut buffer = [0u16; 260];
            let mut image_name = UNICODE_STRING {
                Length: 0,
                MaximumLength: (buffer.len() * 2) as u16,
                Buffer: buffer.as_mut_ptr(),
            };

            let mut info = SYSTEM_PROCESS_ID_INFORMATION {
                ProcessId: pid as HANDLE,
                ImageName: image_name,
            };
            
            let status = NtQuerySystemInformation(
                SystemProcessIdInformation,
                &mut info as *mut _ as *mut _,
                core::mem::size_of::<SYSTEM_PROCESS_ID_INFORMATION>() as u32,
                core::ptr::null_mut(),
            );
            
            if status != STATUS_SUCCESS {
                return U16CStackString::new();
            }
            
            if info.ImageName.Length > 0 && !info.ImageName.Buffer.is_null() {
                let len = info.ImageName.Length as usize / 2;
                let slice = core::slice::from_raw_parts(info.ImageName.Buffer, len);
                let mut result = U16CStackString::<260>::new();
                for &ch in slice {
                    result.push(ch);
                }
                result
            } else {
                U16CStackString::new()
            }
        }
    }
    
    pub fn path_req_admin(&self) -> U16CStackString<260> {
        let pid = self.pid();
        
        unsafe {
            let mut handle = core::ptr::null_mut();
            let mut client_id = CLIENT_ID {
                UniqueProcess: pid as HANDLE,
                UniqueThread: core::ptr::null_mut(),
            };
            let mut attrs: OBJECT_ATTRIBUTES = core::mem::zeroed();

            let status = NtOpenProcess(
                &mut handle,
                PROCESS_QUERY_LIMITED_INFORMATION,
                &mut attrs,
                &mut client_id,
            );

            // let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            
            if status != 0 || handle.is_null() {
                return U16CStackString::new();
            }
            
            let mut return_length = 0;
            let mut buffer = [0u8; 1024];
            
            let status = NtQueryInformationProcess(
                handle,
                ProcessImageFileNameWin32,
                buffer.as_mut_ptr() as *mut _,
                buffer.len() as u32,
                &mut return_length,
            );

            if status != 0 {
                return U16CStackString::new();
            }

            let result = if status == 0 {
                let unicode = buffer.as_ptr() as *const UNICODE_STRING;
                let len = (*unicode).Length as usize / 2;
                let ptr = (*unicode).Buffer;
                
                let slice = core::slice::from_raw_parts(ptr, len);
                let mut path = U16CStackString::<260>::new();
                for &ch in slice {
                    path.push(ch);
                }
                path
            } else {
                let status2 = NtQueryInformationProcess(
                    handle,
                    ProcessImageFileName,
                    buffer.as_mut_ptr() as *mut _,
                    buffer.len() as u32,
                    &mut return_length,
                );
                
                if status2 == 0 {
                    let unicode = buffer.as_ptr() as *const UNICODE_STRING;
                    let len = (*unicode).Length as usize / 2;
                    let ptr = (*unicode).Buffer;
                    
                    let slice = core::slice::from_raw_parts(ptr, len);
                    let mut path = U16CStackString::<260>::new();
                    for &ch in slice {
                        path.push(ch);
                    }
                    path
                } else {
                    U16CStackString::new()
                }
            };
            
            NtClose(handle);
            result
        }
    }
}

impl Iterator for SystemProcessIterator {
    type Item = SystemProcessInfo;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            return None;
        }

        unsafe {
            let mut current_ptr = BUFFER.as_ptr();
            
            for _ in 0..self.index {
                let info = current_ptr as *const SYSTEM_PROCESS_INFORMATION;
                let offset = (*info).NextEntryOffset;
                if offset == 0 {
                    return None;
                }
                current_ptr = current_ptr.add(offset as usize);
            }
            
            let info = current_ptr as *const SYSTEM_PROCESS_INFORMATION;
            let result = *info;
            self.index += 1;
            Some(SystemProcessInfo(result))
        }
    }
}


pub struct ProcessKiller;

impl ProcessKiller {
    pub fn kill_by_handle(handle: HANDLE, exit_code: i32) -> Result<(), NTSTATUS> {
        unsafe {
            let status = NtTerminateProcess(handle, exit_code);

            if status != 0 {
                NtClose(handle);
                return Err(status as _);
            }

            NtClose(handle);
        }
        Ok(())
    }

    pub fn kill_by_pid(pid: u32, exit_code: i32) -> Result<(), NTSTATUS> {
        let handle = ProcessOpener::open_full_access(pid)?;
        ProcessKiller::kill_by_handle(handle, exit_code)
    }

    pub fn kill_current(exit_code: u32) -> ! {
        unsafe {
            let handle = winapi::um::processthreadsapi::GetCurrentProcess();
            TerminateProcess(handle, exit_code);
            NtClose(handle);
        }
        loop {}
    }
}

pub struct ProcessOpener;

impl ProcessOpener {
    pub fn open_full_access(pid: u32) -> Result<HANDLE, NTSTATUS> {
        let mut handle = HANDLE::default();
        let mut client_id = CLIENT_ID {
            UniqueProcess: pid as *mut _,
            UniqueThread: ptr::null_mut(),
        };
        let mut object_attributes = OBJECT_ATTRIBUTES {
            Length: mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: ptr::null_mut(),
            ObjectName: ptr::null_mut(),
            Attributes: 0,
            SecurityDescriptor: ptr::null_mut(),
            SecurityQualityOfService: ptr::null_mut(),
        };

        let status = unsafe {
            NtOpenProcess(
                &mut handle,
                PROCESS_ALL_ACCESS,
                &mut object_attributes,
                &mut client_id,
            )
        };

        if status != STATUS_SUCCESS {
            return Err(status as _);
        }

        Ok(handle)
    }
}

const MAX_BUFFER_SIZE: usize = 0x100000;

#[repr(align(8))]
struct AlignedBuffer([u8; MAX_BUFFER_SIZE]);

impl Deref for AlignedBuffer {
    type Target = [u8; MAX_BUFFER_SIZE];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AlignedBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

static mut PROCESS_BUFFER: AlignedBuffer = AlignedBuffer([0u8; MAX_BUFFER_SIZE]);
static BUFFER_INITIALIZED: AtomicBool = AtomicBool::new(false);

pub struct ProcessEnumerator {
    buffer_size: usize,
    offset: usize,
}

impl ProcessEnumerator {
    pub fn new() -> Self {
        let mut enumerator = ProcessEnumerator {
            buffer_size: 0,
            offset: 0,
        };
        enumerator.query_system_information();
        enumerator
    }

    fn query_system_information(&mut self) {
        let mut return_length: u32 = 0;

        unsafe {
            let buffer_ptr = PROCESS_BUFFER.as_mut_ptr();
            
            let mut buffer_size = MAX_BUFFER_SIZE as u32;
            let mut status = NtQuerySystemInformation(
                SystemProcessInformation as u32,
                buffer_ptr as *mut _,
                buffer_size,
                &mut return_length,
            );

            if status == STATUS_INFO_LENGTH_MISMATCH {
                if return_length > 0 && return_length <= MAX_BUFFER_SIZE as u32 {
                    buffer_size = return_length;
                    status = NtQuerySystemInformation(
                        SystemProcessInformation as u32,
                        buffer_ptr as *mut _,
                        buffer_size,
                        &mut return_length,
                    );
                }
            }

            if status == STATUS_SUCCESS {
                self.buffer_size = return_length as usize;
                BUFFER_INITIALIZED.store(true, Ordering::SeqCst);
            }
        }
    }

    pub fn iter(&self) -> ProcessIterator {
        ProcessIterator {
            buffer: unsafe { &PROCESS_BUFFER[..self.buffer_size] },
            offset: 0,
        }
    }
}

impl Default for ProcessEnumerator {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ProcessIterator<'a> {
    buffer: &'a [u8],
    offset: usize,
}

impl<'a> Iterator for ProcessIterator<'a> {
    type Item = ProcessEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() || self.offset >= self.buffer.len() {
            return None;
        }

        let entry = unsafe {
            &*(self.buffer.as_ptr().add(self.offset) as *const SYSTEM_PROCESS_INFORMATION)
        };

        let process_entry = ProcessEntry::from_process_info(entry);

        if entry.NextEntryOffset == 0 {
            self.offset = self.buffer.len();
        } else {
            self.offset += entry.NextEntryOffset as usize;
        }

        Some(process_entry)
    }
}


pub struct ProcessEntry {
    pub pid: u32,
    pub parent_pid: u32,
    pub name: U8CStackString<260>,
}

impl ProcessEntry {
    pub fn from_process_info(
        info: &SYSTEM_PROCESS_INFORMATION,
    ) -> ProcessEntry {
        let pid = info.UniqueProcessId as u32;
        let parent_pid = info.InheritedFromUniqueProcessId as u32;
        let mut name = U8CStackString::<260>::new();
        
        let image_name = &info.ImageName;
        if !image_name.Buffer.is_null() && image_name.Length > 0 {
            let len = (image_name.Length / 2) as usize;
            let wide_ptr = image_name.Buffer as *const u16;
            let wide_slice = unsafe { core::slice::from_raw_parts(wide_ptr, len) };

            let start_idx = wide_slice.iter().rposition(|&c| c == '\\' as u16 || c == '/' as u16)
                .map(|pos| pos + 1)
                .unwrap_or(0);

            for &c in &wide_slice[start_idx..] {
                if c != 0 && c <= 0xFF {
                    let _ = name.push(c as u8);
                }
            }
        }
        
        if pid == 4 && name.is_empty() {
            let _ = name.push_str("System");
        }

        ProcessEntry {
            pid,
            parent_pid,
            name,
        }
    }
}


pub struct ProcessQuerier;

impl ProcessQuerier {
    pub fn query_peb(process: HANDLE) -> Result<PPEB, NTSTATUS> {
        let mut pbi: PROCESS_BASIC_INFORMATION = unsafe { mem::zeroed() };
        let mut return_length = 0;

        let status = unsafe {
            NtQueryInformationProcess(
                process,
                ProcessBasicInformation,
                &mut pbi as *mut _ as *mut _,
                mem::size_of::<PROCESS_BASIC_INFORMATION>() as u32,
                &mut return_length,
            )
        };

        if status != STATUS_SUCCESS {
            return Err(status);
        }

        Ok(pbi.PebBaseAddress)
    }

    pub fn find_process_by_name<const N: usize>(
        name: &U8CStackString<N>,
    ) -> Option<u32> {
        let name_bytes = name.as_slice();
        let enumerator = ProcessEnumerator::new();

        for entry in enumerator.iter() {
            let entry_name = entry.name.as_slice();
            if entry_name.len() >= name_bytes.len() {
                let matches = name_bytes.iter()
                    .zip(entry_name.iter())
                    .all(|(a, b)| a == b);
                if matches {
                    return Some(entry.pid);
                }
            }
        }

        None
    }

    pub fn get_process_name_by_pid<const N: usize>(pid: u32) -> Option<U8CStackString<N>> {
        let enumerator = ProcessEnumerator::new();
        for entry in enumerator.iter() {
            if entry.pid == pid {
                let name_slice = entry.name.as_slice();
                return U8CStackString::from_bytes(name_slice);
            }
        }
        None
    }

    pub fn get_all_processes() -> ProcessEnumerator {
        ProcessEnumerator::new()
    }
}

pub struct ProcessInfo {
    pub handle: HANDLE,
    pub thread: HANDLE,
    pub pid: u32,
    pub tid: u32,
}

impl ProcessInfo {
    pub fn from_inner(inner: PROCESS_INFORMATION) -> Self {
        ProcessInfo {
            handle: inner.hProcess,
            thread: inner.hThread,
            pid: inner.dwProcessId,
            tid: inner.dwThreadId,
        }
    }
}

impl Drop for ProcessInfo {
    fn drop(&mut self) {
        unsafe {
            if !self.handle.is_null() {
                CloseHandle(self.handle);
            }
            if !self.thread.is_null() {
                CloseHandle(self.thread);
            }
        }
    }
}

pub struct ProcessSpawner;

impl ProcessSpawner {
    pub fn create_suspended<const N: usize>(path: U16CStackString<N>) -> Result<ProcessInfo, NTSTATUS> {
        let mut startup_info: STARTUPINFOW = unsafe { mem::zeroed() };
        startup_info.cb = mem::size_of::<STARTUPINFOW>() as u32;
        let mut process_info: PROCESS_INFORMATION = unsafe { mem::zeroed() };

        let result = unsafe {
            CreateProcessW(
                path.as_ptr(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                FALSE,
                // CREATE_SUSPENDED | DETACHED_PROCESS,
                CREATE_SUSPENDED,
                ptr::null_mut(),
                ptr::null_mut(),
                &mut startup_info,
                &mut process_info,
            )
        };

        if result == 0 {
            return Err(unsafe { GetLastError() as _});
        }

        let info = ProcessInfo::from_inner(process_info);
        Ok(info)
    }
}