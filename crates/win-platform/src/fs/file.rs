use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::ptr::{NonNull, null_mut};
use core::mem::size_of;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

use crate::types::UnicodeString;

use crate::types::HANDLE;

use ntapi::ntioapi::{FILE_BASIC_INFORMATION, FILE_NON_DIRECTORY_FILE, FILE_POSITION_INFORMATION, FILE_STANDARD_INFORMATION, FileBasicInformation, FilePositionInformation, FileStandardInformation, IO_STATUS_BLOCK};
use winapi::shared::ntdef::{OBJ_CASE_INSENSITIVE, OBJECT_ATTRIBUTES, UNICODE_STRING};
use winapi::um::winnt::LARGE_INTEGER;

use crate::fs::options::FileOptions;
use crate::fs::types::*;
use crate::{io::*};

use crate::syscalls::*;
use crate::fs::*;

pub struct File {
    handle: HANDLE,
    offset: u64,
}

unsafe impl Send for File {}
unsafe impl Sync for File {}

impl File {

    pub fn open(mut path_uc: UnicodeString) -> Result<Self, FileError> {
        let mut opts = FileOptions::new();
        opts.read().share_read().synchronous();
        Self::open_with_options(path_uc, &opts)
    }

    pub fn create(path_uc: UnicodeString) -> Result<Self, FileError> {
        let mut opts = FileOptions::new();
        opts
            .read_write()
            .share_all()
            .truncate_always()
            .synchronous();
        Self::create_with_options(path_uc, &opts)
    }

    pub fn open_with_options(mut path_uc: UnicodeString, opts: &FileOptions) -> Result<Self, FileError> {
        let mut path_uc_raw = path_uc.as_unicode_string();
        let mut object_attributes = OBJECT_ATTRIBUTES {
            Length: size_of::<OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: null_mut(),
            ObjectName: &mut path_uc_raw,
            Attributes: OBJ_CASE_INSENSITIVE,
            SecurityDescriptor: null_mut(),
            SecurityQualityOfService: null_mut(),
        };
        
        let mut handle: HANDLE = null_mut();
        let mut status_block: IO_STATUS_BLOCK = unsafe { core::mem::zeroed() };
        
        let (access, share, create_options, _, _) = opts.build();
        let create_options = create_options | FILE_NON_DIRECTORY_FILE;
        
        NtOpenFile(
            &mut handle,
            access,
            &mut object_attributes,
            &mut status_block,
            share,
            create_options,
        ).ok()?;
        
        Ok(Self { handle, offset: 0 })
    }

    pub fn create_with_options(mut path_uc: UnicodeString, opts: &FileOptions) -> Result<Self, FileError> {
        let mut path_uc_raw = path_uc.as_unicode_string();
        let mut object_attributes = OBJECT_ATTRIBUTES {
            Length: size_of::<OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: null_mut(),
            ObjectName: &mut path_uc_raw,
            Attributes: OBJ_CASE_INSENSITIVE,
            SecurityDescriptor: null_mut(),
            SecurityQualityOfService: null_mut(),
        };
        
        let mut handle: HANDLE = null_mut();
        let mut status_block: IO_STATUS_BLOCK = unsafe { core::mem::zeroed() };

        let (access, share, create_options, disposition, attributes) = opts.build();

        NtCreateFile(
            &mut handle,
            access,
            &mut object_attributes,
            &mut status_block,
            null_mut(),
            attributes,
            share,
            disposition,
            create_options,
            null_mut(),
            0,
        ).ok()?;

        Ok(Self { handle, offset: 0 })
    }

    // #[cfg(feature = "alloc")]
    // pub fn open_with_flags<P: AsRef<Path>>(path: P, access: u32, share: u32) -> Result<Self, FileError> {
    //     let mut opts = FileOptions::new();
    //     opts.access = access;
    //     opts.share = share;
    //     opts.create_options = FILE_SYNCHRONOUS_IO_NONALERT | FILE_NON_DIRECTORY_FILE;
    //     Self::open_with_options(path, &opts)
    // }

    pub fn metadata(&self) -> Result<FileMetadata, FileError> {
        unsafe {
            let mut standard_info: FILE_STANDARD_INFORMATION = core::mem::zeroed();
            let mut status_block: IO_STATUS_BLOCK = core::mem::zeroed();
            
            NtQueryInformationFile(
                self.handle as _,
                &mut status_block,
                &mut standard_info as *mut _ as _,
                size_of::<FILE_STANDARD_INFORMATION>() as u32,
                FileStandardInformation,
            ).ok()?;
            
            let mut basic_info: FILE_BASIC_INFORMATION = core::mem::zeroed();
            NtQueryInformationFile(
                self.handle as _,
                &mut status_block,
                &mut basic_info as *mut _ as _,
                size_of::<FILE_BASIC_INFORMATION>() as u32,
                FileBasicInformation,
            ).ok()?;
            
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

    pub fn handle(&self) -> HANDLE {
        self.handle
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
        ).ok()?;
        
        let read = status_block.Information as usize;
        self.offset += read as u64;
        Ok(read)
    }
}


impl core::io::Write for File {
    fn write(&mut self, buf: &[u8]) -> core::io::Result<usize> {
        let mut status_block: IO_STATUS_BLOCK = unsafe { core::mem::zeroed() };

        let mut byte_offset: LARGE_INTEGER = unsafe { core::mem::zeroed() };
        unsafe { *byte_offset.QuadPart_mut() = self.offset as i64; }

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
        ).ok()?;
        
        let written = status_block.Information as usize;
        self.offset += written as u64;
        Ok(written)
    }

    fn flush(&mut self) -> core::io::Result<()> {
        let mut status_block: IO_STATUS_BLOCK = unsafe { core::mem::zeroed() };
        
        NtFlushBuffersFile(
            self.handle,
            &mut status_block,
        ).ok()?;

        Ok(())
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> Result<usize, FileError> {
        let mut status_block: IO_STATUS_BLOCK = unsafe { core::mem::zeroed() };

        let mut byte_offset: LARGE_INTEGER = unsafe { core::mem::zeroed() };
        unsafe { *byte_offset.QuadPart_mut() = self.offset as i64; }

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
        ).ok()?;

        let written = status_block.Information as usize;
        self.offset += written as u64;

        Ok(written)
    }
}

impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, FileError> {
        unsafe {
            let current_pos = {
                let mut pos_info: FILE_POSITION_INFORMATION = core::mem::zeroed();
                let mut status_block: IO_STATUS_BLOCK = core::mem::zeroed();
                
                NtQueryInformationFile(
                    self.handle,
                    &mut status_block,
                    &mut pos_info as *mut _ as _,
                    size_of::<FILE_POSITION_INFORMATION>() as u32,
                    FilePositionInformation,
                ).ok()?;

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
            
            NtSetInformationFile(
                self.handle,
                &mut status_block,
                &mut pos_info as *mut _ as _,
                size_of::<FILE_POSITION_INFORMATION>() as u32,
                FilePositionInformation,
            );
            
            self.offset = new_pos;

            Ok(new_pos)
        }
    }
}