use core::ptr::null_mut;
use core::mem::size_of;
use ntapi::ntioapi::{
    FILE_BASIC_INFORMATION, FILE_NON_DIRECTORY_FILE, FILE_OVERWRITE_IF, FILE_POSITION_INFORMATION, FILE_STANDARD_INFORMATION, FILE_SYNCHRONOUS_IO_NONALERT, FileBasicInformation, FilePositionInformation, FileStandardInformation, IO_STATUS_BLOCK, NtCreateFile, NtOpenFile, NtQueryInformationFile, NtReadFile, NtSetInformationFile, NtWriteFile,
};
use ntapi::ntobapi::NtClose;
use winapi::shared::ntdef::{HANDLE, OBJECT_ATTRIBUTES, UNICODE_STRING, OBJ_CASE_INSENSITIVE};
use winapi::um::winnt::{FILE_READ_ATTRIBUTES, FILE_READ_DATA, FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_WRITE_DATA, LARGE_INTEGER, SYNCHRONIZE};
use utils::{U16CStackString, println};

use crate::error::FileError;
use crate::fs::options::FileOptions;
use crate::fs::*;
use crate::io::*;

pub struct File {
    handle: HANDLE,
    offset: u64,
}

impl File {
    pub fn open(path: &str) -> Result<Self, FileError> {
        let mut opts = FileOptions::new();
        opts.read().share_read().synchronous();
        Self::open_with_options(path, &opts)
    }

    pub fn create(path: &str) -> Result<Self, FileError> {
        let mut opts = FileOptions::new();
        opts.read_write().share_all().truncate_always().synchronous();
        Self::create_with_options(path, &opts)
    }

    pub fn open_with_options(path: &str, opts: &FileOptions) -> Result<Self, FileError> {
        let mut path = U16CStackString::<260>::from_str(path).ok_or_else(|| FileError::InvalidParameter)?;
        let mut path_uc = path.to_unicode_string();

        let mut object_attributes = OBJECT_ATTRIBUTES {
            Length: size_of::<OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: null_mut(),
            ObjectName: &mut path_uc,
            Attributes: OBJ_CASE_INSENSITIVE,
            SecurityDescriptor: null_mut(),
            SecurityQualityOfService: null_mut(),
        };
        
        let mut handle: HANDLE = null_mut();
        let mut status_block: IO_STATUS_BLOCK = unsafe { core::mem::zeroed() };
        
        let (access, share, create_options, _, _) = opts.build();
        let create_options = create_options | FILE_NON_DIRECTORY_FILE;
        
        let status = unsafe {
            NtOpenFile(
                &mut handle,
                access,
                &mut object_attributes,
                &mut status_block,
                share,
                create_options,
            )
        };
        
        if status >= 0 {
            Ok(Self { handle, offset: 0 })
        } else {
            Err(FileError::from(status))
        }
    }

    pub fn create_with_options(path: &str, opts: &FileOptions) -> Result<Self, FileError> {
        let mut path = U16CStackString::<260>::from_str(path).ok_or_else(|| FileError::InvalidParameter)?;
        let mut path_uc = path.to_unicode_string();
        
        let mut object_attributes = OBJECT_ATTRIBUTES {
            Length: size_of::<OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: null_mut(),
            ObjectName: &mut path_uc,
            Attributes: OBJ_CASE_INSENSITIVE,
            SecurityDescriptor: null_mut(),
            SecurityQualityOfService: null_mut(),
        };
        
        let mut handle: HANDLE = null_mut();
        let mut status_block: IO_STATUS_BLOCK = unsafe { core::mem::zeroed() };

        let (access, share, create_options, disposition, attributes) = opts.build();
        
        let status = unsafe {
            NtCreateFile(
                &mut handle,
                access,
                &mut object_attributes,
                &mut status_block,
                null_mut(),
                attributes,
                0,
                disposition,
                create_options,
                null_mut(),
                0,
            )
        };
        
        if status >= 0 {
            Ok(Self { handle, offset: 0 })
        } else {
            Err(FileError::from(status))
        }
    }

    pub fn open_with_flags(path: &str, access: u32, share: u32) -> Result<Self, FileError> {
        let mut opts = FileOptions::new();
        opts.access = access;
        opts.share = share;
        opts.create_options = FILE_SYNCHRONOUS_IO_NONALERT | FILE_NON_DIRECTORY_FILE;
        Self::open_with_options(path, &opts)
    }


    pub fn metadata(&self) -> Result<FileMetadata, FileError> {
        unsafe {
            let mut standard_info: FILE_STANDARD_INFORMATION = core::mem::zeroed();
            let mut status_block: IO_STATUS_BLOCK = core::mem::zeroed();
            
            let status = NtQueryInformationFile(
                self.handle,
                &mut status_block,
                &mut standard_info as *mut _ as _,
                size_of::<FILE_STANDARD_INFORMATION>() as u32,
                FileStandardInformation,
            );
            
            if status < 0 {
                return Err(FileError::from(status));
            }
            
            let mut basic_info: FILE_BASIC_INFORMATION = core::mem::zeroed();
            let status = unsafe {
                NtQueryInformationFile(
                    self.handle,
                    &mut status_block,
                    &mut basic_info as *mut _ as _,
                    size_of::<FILE_BASIC_INFORMATION>() as u32,
                    FileBasicInformation,
                )
            };
            
            if status < 0 {
                return Err(FileError::from(status));
            }
            
            let attrs = basic_info.FileAttributes;
            Ok(FileMetadata {
                size: FileSize(*standard_info.EndOfFile.QuadPart() as u64),
                creation_time: FileTime(*basic_info.CreationTime.QuadPart() as u64),
                last_access_time: FileTime(*basic_info.LastAccessTime.QuadPart() as u64),
                last_write_time: FileTime(*basic_info.LastWriteTime.QuadPart() as u64),
                attributes: FileAttributes(attrs),
            })
        }
    }

    pub fn into_handle(self) -> HANDLE {
        let handle = self.handle;
        core::mem::forget(self);
        handle
    }
}

impl Drop for File {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { NtClose(self.handle); }
        }
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, FileError> {
        let mut status_block: IO_STATUS_BLOCK = unsafe { core::mem::zeroed() };
        
        let status = unsafe {
            NtReadFile(
                self.handle,
                null_mut(),
                None,
                null_mut(),
                &mut status_block,
                buf.as_mut_ptr() as _,
                buf.len() as u32,
                null_mut(),
                null_mut(),
            )
        };
        
        if status >= 0 || status == 0x80000005u32 as _ {
            let read = status_block.Information as usize;
            self.offset += read as u64;
            Ok(read)
        } else {
            Err(FileError::from(status))
        }
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> Result<usize, FileError> {
        let mut status_block: IO_STATUS_BLOCK = unsafe { core::mem::zeroed() };

        let mut byte_offset: LARGE_INTEGER = unsafe { core::mem::zeroed() };
        unsafe { *byte_offset.QuadPart_mut() = self.offset as i64; }

        let status = unsafe {
            NtWriteFile(
                self.handle,
                null_mut(),
                None,
                null_mut(),
                &mut status_block,
                buf.as_ptr() as _,
                buf.len() as u32,
                &mut byte_offset,
                null_mut(),
            )
        };
        
        if status >= 0 {
            let written = status_block.Information as usize;
            if written == 0 {
                println!("NtWriteFile {:X} {}", status, buf.len());
            }
            self.offset += written as u64;
            Ok(written)
        } else {
            Err(FileError::from(status))
        }
    }
}

impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, FileError> {
        unsafe {
            let current_pos = {
            let mut pos_info: FILE_POSITION_INFORMATION = core::mem::zeroed();
            let mut status_block: IO_STATUS_BLOCK = core::mem::zeroed();
            
            let status = NtQueryInformationFile(
                self.handle,
                &mut status_block,
                &mut pos_info as *mut _ as _,
                size_of::<FILE_POSITION_INFORMATION>() as u32,
                FilePositionInformation,
            );
            
            if status < 0 {
                    return Err(FileError::from(status));
                }
                *pos_info.CurrentByteOffset.QuadPart() as u64
            };
            
            let new_pos = match pos {
                SeekFrom::Start(offset) => offset,
                SeekFrom::End(offset) => {
                    let metadata = self.metadata()?;
                    let size = metadata.size.0;
                    if offset >= 0 {
                        size.checked_add(offset as u64).ok_or(FileError::InvalidParameter)?
                    } else {
                        size.checked_sub((-offset) as u64).ok_or(FileError::InvalidParameter)?
                    }
                }
                SeekFrom::Current(offset) => {
                    if offset >= 0 {
                        current_pos.checked_add(offset as u64).ok_or(FileError::InvalidParameter)?
                    } else {
                        current_pos.checked_sub((-offset) as u64).ok_or(FileError::InvalidParameter)?
                    }
                }
            };
            
            let mut pos_info = FILE_POSITION_INFORMATION {
                CurrentByteOffset: core::mem::transmute(new_pos)
            };
            let mut status_block: IO_STATUS_BLOCK = core::mem::zeroed();
            
            let status = NtSetInformationFile(
                self.handle,
                &mut status_block,
                &mut pos_info as *mut _ as _,
                size_of::<FILE_POSITION_INFORMATION>() as u32,
                FilePositionInformation,
            );
            
            if status >= 0 {
                self.offset = new_pos;
                Ok(new_pos)
            } else {
                Err(FileError::from(status))
            }
        }
    }
}