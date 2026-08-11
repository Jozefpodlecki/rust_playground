use core::mem;
use ntapi::{ntexapi::*, ntldr::{RTL_PROCESS_MODULE_INFORMATION, RTL_PROCESS_MODULES}};
use winapi::shared::{ntdef::NTSTATUS, ntstatus::STATUS_SUCCESS};

const MODULE_BUFFER_SIZE: usize = 2_000_000;
static mut MODULE_BUFFER: [u8; MODULE_BUFFER_SIZE] = [0; MODULE_BUFFER_SIZE];

pub struct SystemModuleIterator {
    index: usize,
    count: usize,
}

impl SystemModuleIterator {
    pub fn new() -> Result<Self, NTSTATUS> {
        unsafe {
            let mut return_length = 0;

            let status = NtQuerySystemInformation(
                SystemModuleInformation,
                MODULE_BUFFER.as_mut_ptr() as *mut _,
                MODULE_BUFFER_SIZE as u32,
                &mut return_length,
            );

            if status != STATUS_SUCCESS {
                return Err(status);
            }

            let modules = MODULE_BUFFER.as_ptr() as *const RTL_PROCESS_MODULES;
            let count = (*modules).NumberOfModules as usize;

            Ok(SystemModuleIterator {
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
pub struct SystemModuleInfo(RTL_PROCESS_MODULE_INFORMATION);

impl SystemModuleInfo {
    pub fn new(value: RTL_PROCESS_MODULE_INFORMATION) -> Self {
        Self(value)
    }

    pub fn image_base(&self) -> *mut winapi::ctypes::c_void {
        self.0.ImageBase
    }

    pub fn image_size(&self) -> u32 {
        self.0.ImageSize
    }

    pub fn file_path(&self) -> ModuleFilePath {
        let path_bytes = self.0.FullPathName;
        let offset = self.0.OffsetToFileName as usize;
        
        ModuleFilePath {
            buffer: path_bytes,
            offset_to_filename: offset,
        }
    }
}

pub struct ModuleFilePath {
    buffer: [u8; 256],
    offset_to_filename: usize,
}

impl ModuleFilePath {
    pub fn full_path(&self) -> &str {
        let len = self.buffer.iter().position(|&b| b == 0).unwrap_or(256);
        unsafe { core::str::from_utf8_unchecked(&self.buffer[..len]) }
    }

    pub fn file_name(&self) -> &str {
        let full = self.full_path();
        let start = self.offset_to_filename.min(full.len());
        &full[start..]
    }
}

impl Iterator for SystemModuleIterator {
    type Item = SystemModuleInfo;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            return None;
        }

        unsafe {
            let modules = MODULE_BUFFER.as_ptr() as *const RTL_PROCESS_MODULES;
            let module_size = mem::size_of::<RTL_PROCESS_MODULE_INFORMATION>();
            let offset = mem::offset_of!(RTL_PROCESS_MODULES, Modules);
            
            let current_ptr = MODULE_BUFFER.as_ptr().add(offset + (self.index * module_size));
            let entry = current_ptr.cast::<RTL_PROCESS_MODULE_INFORMATION>().read_unaligned();
            
            self.index += 1;
            Some(SystemModuleInfo(entry))
        }
    }
}