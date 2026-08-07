use core::fmt::{self, Write};
use core::marker::PhantomData;
use core::ptr::null_mut;
use core::mem::size_of;
use ntapi::ntioapi::{FILE_BASIC_INFORMATION, FILE_CREATE, FILE_DIRECTORY_FILE, FILE_DIRECTORY_INFORMATION, FILE_NON_DIRECTORY_FILE, FILE_STANDARD_INFORMATION, FILE_SYNCHRONOUS_IO_NONALERT, FileBasicInformation, FileDirectoryInformation, FileStandardInformation, IO_STATUS_BLOCK, NtCreateFile};
use winapi::shared::ntdef::{HANDLE, OBJ_CASE_INSENSITIVE, OBJECT_ATTRIBUTES, UNICODE_STRING};
use winapi::shared::ntstatus::STATUS_NO_MORE_FILES;
use winapi::um::winnt::{FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_NORMAL, FILE_ATTRIBUTE_REPARSE_POINT, FILE_LIST_DIRECTORY, FILE_SHARE_READ, FILE_SHARE_WRITE, SYNCHRONIZE};
use crate::error::FileError;
use crate::fs::options::FileOptions;
use crate::{FileAttributes, FileMetadata, FileName, FilePath, FileSize, FileTime, println};
use crate::syscalls::{NtClose, NtOpenFile, NtQueryDirectoryFile, NtQueryInformationFile};
use crate::types::ToUnicode;

pub struct FullPath {
    data: [u16; 260],
    len: usize,
}

impl FullPath {
    pub fn new(parent: &[u16], name: &[u16]) -> Self {
        let mut data = [0u16; 260];
        let mut pos = 0;
        
        for &c in parent {
            if pos < 259 {
                data[pos] = c;
                pos += 1;
            }
        }
        
        if pos > 0 && data[pos - 1] != b'\\' as u16 {
            if pos < 259 {
                data[pos] = b'\\' as u16;
                pos += 1;
            }
        }
        
        for &c in name {
            if pos < 259 {
                data[pos] = c;
                pos += 1;
            }
        }
        
        Self { data, len: pos }
    }
    
    pub fn as_slice(&self) -> &[u16] {
        &self.data[..self.len]
    }
    
    pub fn as_mut_slice(&mut self) -> &mut [u16] {
        &mut self.data[..self.len]
    }

    pub fn as_mut_ptr(&mut self) -> *mut u16 {
        self.data.as_mut_ptr()
    }
    
    pub fn as_ptr(&self) -> *const u16 {
        self.data.as_ptr()
    }
    
    pub fn len(&self) -> usize {
        self.len
    }
    
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

pub struct FileDirectoryInfo<'a>  {
    parent: &'a Directory,
    info: *const FILE_DIRECTORY_INFORMATION
}

impl<'a> FileDirectoryInfo<'a> {
    pub const fn from_buf(ptr: *const FILE_DIRECTORY_INFORMATION, parent: &'a Directory) -> Self {
        Self {
            parent,
            info: ptr,
        }
    }

    pub fn name(&self) -> &[u16] {
        unsafe { 
            let entry = &*self.info;
            core::slice::from_raw_parts(entry.FileName.as_ptr(), entry.FileNameLength as usize / 2)
        }
    }

    fn build_full_path(&self) -> FullPath {
        let parent = self.parent.path();
        let name = self.name();
        FullPath::new(parent, name)
    }

    pub fn open_with_options(&self, opts: FileOptions) -> Result<HANDLE, FileError> {
        let mut full_path = self.build_full_path();
        let mut path_uc = UNICODE_STRING {
            Length: (full_path.len() * 2) as u16,
            MaximumLength: 260 * 2,
            Buffer: full_path.as_mut_ptr(),
        };
        
        // let mut opts = FileOptions::new();
        // opts.read().share_read().synchronous();

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
        let create_options = if self.is_directory() {
            create_options | FILE_LIST_DIRECTORY
        } else {
            create_options | FILE_NON_DIRECTORY_FILE
        };
        
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
                    println!("0x{status:X}");
        if status >= 0 {
            Ok(handle)
        } else {
            Err(FileError::from(status))
        }
    }

    pub fn metadata(&self) -> Result<FileMetadata, FileError> {
        unsafe {
            let mut standard_info: FILE_STANDARD_INFORMATION = core::mem::zeroed();
            let mut status_block: IO_STATUS_BLOCK = core::mem::zeroed();
            let mut opts = FileOptions::new();
            opts.attributes_only().share_all().synchronous();
            let handle = self.open_with_options(opts)?;

            let status = NtQueryInformationFile(
                handle,
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
                    handle,
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

    pub fn next_entry_offset(&self) -> u32 {
        unsafe { (*self.info).NextEntryOffset }
    }
    
    pub fn file_index(&self) -> u32 {
        unsafe { (*self.info).FileIndex }
    }
    
    pub fn creation_time(&self) -> i64 {
        unsafe { *(*self.info).CreationTime.QuadPart() }
    }
    
    pub fn last_access_time(&self) -> i64 {
        unsafe { *(*self.info).LastAccessTime.QuadPart() }
    }
    
    pub fn last_write_time(&self) -> i64 {
        unsafe { *(*self.info).LastWriteTime.QuadPart() }
    }
    
    pub fn change_time(&self) -> i64 {
        unsafe { *(*self.info).ChangeTime.QuadPart() }
    }
    
    pub fn end_of_file(&self) -> FileSize {
        unsafe { FileSize(*(*self.info).EndOfFile.QuadPart() as u64) }
    }
    
    pub fn allocation_size(&self) -> i64 {
        unsafe { *(*self.info).AllocationSize.QuadPart() }
    }
    
    pub fn file_attributes(&self) -> FileAttributes {
        unsafe { FileAttributes((*self.info).FileAttributes) }
    }
    
    pub fn file_name_length(&self) -> u32 {
        unsafe { (*self.info).FileNameLength }
    }
    
    pub fn file_name(&self) -> FileName<'a> {
        let entry = unsafe { &*self.info };
        let name_len = entry.FileNameLength as usize / 2;
        unsafe {
            FileName(core::slice::from_raw_parts(
                entry.FileName.as_ptr(),
                name_len
            ))
        }
    }

    pub fn file_path(&self) -> FilePath<260> {
        let entry = unsafe { &*self.info };
        let name_len = entry.FileNameLength as usize / 2;
        let name_ptr = entry.FileName.as_ptr();
        let parent_path = self.parent.path();
        
        unsafe {
            let name = core::slice::from_raw_parts(name_ptr, name_len);
            FilePath::new(parent_path, name)
        }
    }

    pub fn is_file(&self) -> bool {
        unsafe { (*self.info).FileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0 }
    }

    pub fn is_directory(&self) -> bool {
        unsafe { (*self.info).FileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0 }
    }

    pub fn is_reparse_point(&self) -> bool {
        unsafe { (*self.info).FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 }
    }

    pub fn is_junction(&self) -> bool {
        if !self.is_reparse_point() {
            return false;
        }

        self.is_directory()
    }

    pub fn is_symlink(&self) -> bool {
        if !self.is_reparse_point() {
            return false;
        }

        true
    }

    pub fn is_mount_point(&self) -> bool {
        if !self.is_reparse_point() {
            return false;
        }
        self.is_directory()
    }

    pub fn is_self(&self) -> bool {
        self.file_name().is_self()
    }

    pub fn is_parent(&self) -> bool {
        self.file_name().is_parent()
    }
}

pub struct Directory {
    path_len: u16,
    path_buf: [u16; 260],
    handle: HANDLE
}

impl Directory {

     pub fn create(path_slice: &[u16]) -> Result<Self, FileError> {
        let mut handle: HANDLE = null_mut();
        let path_len = path_slice.len();
        let mut path_buf: [u16; 260] = [0; 260];
        path_buf[..path_len].copy_from_slice(&path_slice);
        path_buf[path_len] = 0; // Null terminate

        let mut path_uc = UNICODE_STRING {
            Length: (path_len * 2) as u16,
            MaximumLength: ((path_len + 1) * 2) as u16,
            Buffer: path_buf.as_mut_ptr(),
        };

        let mut object_attributes = OBJECT_ATTRIBUTES {
            Length: size_of::<OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: null_mut(),
            ObjectName: &mut path_uc,
            Attributes: OBJ_CASE_INSENSITIVE,
            SecurityDescriptor: null_mut(),
            SecurityQualityOfService: null_mut(),
        };

        let mut io_status_block: IO_STATUS_BLOCK = unsafe { core::mem::zeroed() };

        let status = unsafe {
            NtCreateFile(
                &mut handle,
                FILE_LIST_DIRECTORY | SYNCHRONIZE,
                &mut object_attributes,
                &mut io_status_block,
                null_mut(),
                FILE_ATTRIBUTE_NORMAL,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                FILE_CREATE,
                FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT,
                null_mut(),
                0,
            )
        };

        if status >= 0 {
            Ok(Directory {
                handle,
                path_len: path_uc.Length,
                path_buf,
            })
        } else {
            Err(FileError::from(status))
        }
    }

    pub fn open(path_slice: &[u16]) -> Result<Self, FileError> {
        let mut handle: HANDLE = null_mut();

        let path_len = path_slice.len();
        let mut path_buf: [u16; 260] = [0; 260];
        path_buf[..path_len].copy_from_slice(&path_slice);

        let mut path_uc = UNICODE_STRING {
            Length: (path_len * 2) as u16,
            MaximumLength: (path_len * 2) as u16 + 2,
            Buffer: path_buf.as_mut_ptr(),
        };

        let mut object_attributes = OBJECT_ATTRIBUTES {
            Length: size_of::<OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: null_mut(),
            ObjectName: &mut path_uc,
            Attributes: OBJ_CASE_INSENSITIVE,
            SecurityDescriptor: null_mut(),
            SecurityQualityOfService: null_mut(),
        };
        let mut status_block: IO_STATUS_BLOCK = unsafe { core::mem::zeroed() };

        let status = unsafe {
            NtOpenFile(
                &mut handle,
                FILE_LIST_DIRECTORY | SYNCHRONIZE,
                &mut object_attributes,
                &mut status_block,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                FILE_SYNCHRONOUS_IO_NONALERT,
            )
        };

        if status >= 0 {
            Ok(Directory {
                handle,
                path_len: path_uc.Length,
                path_buf: path_buf
            })
        } else {
            Err(FileError::from(status))
        }
    }

    pub fn path(&self) -> &[u16] {
        &self.path_buf[..self.path_len as usize / 2]
    }

    pub fn query(&self) -> DirectoryQuery<'_> {
        DirectoryQuery {
            parent: self,
            buffer: [0u8; 8192],
            position: 0,
            bytes_returned: 0,
            restart_scan: 1,
            done: false,
        }
    }
}

impl Drop for Directory {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { NtClose(self.handle); }
        }
    }
}

pub struct DirectoryQuery<'a> {
    parent: &'a Directory,
    buffer: [u8; 8192],
    position: usize,
    bytes_returned: usize,
    restart_scan: u8,
    done: bool,
}

impl<'a> DirectoryQuery<'a> {
    fn query_next(&mut self) -> Result<bool, FileError> {
        let mut io_status_block: IO_STATUS_BLOCK = unsafe { core::mem::zeroed() };
        let status = unsafe {
            NtQueryDirectoryFile(
                self.parent.handle,
                null_mut(),
                None,
                null_mut(),
                &mut io_status_block,
                self.buffer.as_mut_ptr() as _,
                self.buffer.len() as u32,
                FileDirectoryInformation,
                0,
                null_mut(),
                self.restart_scan,
            )
        };

        if status == STATUS_NO_MORE_FILES {
            self.done = true;
            return Ok(false);
        }

        if status < 0 {
            return Err(FileError::from(status));
        }

        self.restart_scan = 0;
        self.bytes_returned = io_status_block.Information as usize;
        self.position = 0;
        Ok(true)
    }
}

impl<'a>  Iterator for DirectoryQuery<'a>  {
    type Item = Result<FileDirectoryInfo<'a>, FileError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.done {
                return None;
            }

            if self.position >= self.bytes_returned {
                match self.query_next() {
                    Ok(has_more) => {
                        if !has_more {
                            return None;
                        }
                        continue;
                    }
                    Err(e) => return Some(Err(e)),
                }
            }

            let entry_ptr = unsafe { self.buffer.as_ptr().add(self.position) as *const FILE_DIRECTORY_INFORMATION };
            let entry = FileDirectoryInfo::from_buf(entry_ptr, self.parent);
            let skip = entry.is_self() || entry.is_parent();
            let offset = entry.next_entry_offset();
            self.position += offset as usize;

            if offset == 0 {
                self.done = true;
            }

            if self.position == 0 {
                self.done = true;
                return None;
            }

            if !skip {
                return Some(Ok(entry));
            }
        }
    }
}