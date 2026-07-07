
use ntapi::{ntapi_base::CLIENT_ID, ntexapi::*, ntobapi::NtClose, ntpsapi::{NtOpenProcess, NtQueryInformationProcess, ProcessImageFileName, ProcessImageFileNameWin32}};
use crate::{U16CStackString, println};
use winapi::{shared::{ntdef::{HANDLE, NTSTATUS, OBJECT_ATTRIBUTES, UNICODE_STRING}, ntstatus::STATUS_SUCCESS}, um::winnt::PROCESS_QUERY_LIMITED_INFORMATION };

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
pub struct ProcessInfo(pub (crate) SYSTEM_PROCESS_INFORMATION);

impl ProcessInfo {
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
    type Item = ProcessInfo;

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
            Some(ProcessInfo(result))
        }
    }
}
