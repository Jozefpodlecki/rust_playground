use core::ptr::null_mut;
use core::mem::size_of;
use alloc::string::String;
use alloc::vec::Vec;
use ntapi::ntioapi::{FILE_OPEN_BY_FILE_ID, FILE_OPEN_FOR_BACKUP_INTENT, FILE_STANDARD_INFORMATION, FILE_SYNCHRONOUS_IO_NONALERT, FileStandardInformation, IO_STATUS_BLOCK, NtDeviceIoControlFile, NtOpenFile};
use winapi::shared::ntdef::{HANDLE, OBJ_CASE_INSENSITIVE, OBJECT_ATTRIBUTES, UNICODE_STRING, LARGE_INTEGER};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::fileapi::{CreateFileW, OPEN_EXISTING};
use winapi::um::ioapiset::DeviceIoControl;
use winapi::um::winnt::{FILE_ATTRIBUTE_NORMAL, FILE_READ_ATTRIBUTES, FILE_READ_DATA, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE, SYNCHRONIZE};
use winapi::um::winioctl::{FSCTL_ENUM_USN_DATA, FSCTL_QUERY_USN_JOURNAL};
use toolkit::syscalls::{NtClose, NtQueryInformationFile};
use toolkit::{U16CStackString, println};

use crate::types::*;
use crate::utils::*;

pub struct VolumeHandle(HANDLE);

impl Drop for VolumeHandle {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { NtClose(self.0); }
        }
    }
}

// pub struct UsnJournalIter {
//     handle: VolumeHandle,
//     buffer: [u8; 65536],
//     enum_data: MFT_ENUM_DATA,
//     offset: usize,
//     bytes_returned: usize,
//     done: bool,
//     next_start: u64,
// }

// impl UsnJournalIter {
//     pub fn new(volume_path: &[u16]) -> Result<Self, &'static str> {
//         let handle = open_volume(volume_path)?;
//         println!("open_volume {:p}", handle.0);
//         let journal_data = query_usn_journal(&handle)?;
        
//         Ok(Self {
//             handle,
//             buffer: [0u8; 65536],
//             enum_data: MFT_ENUM_DATA {
//                 StartFileReferenceNumber: 0,
//                 LowUsn: 0,
//                 HighUsn: journal_data.MaxUsn,
//             },
//             offset: 0,
//             bytes_returned: 0,
//             done: false,
//             next_start: 0,
//         })
//     }

//     fn refill_buffer(&mut self) -> Result<bool, &'static str> {
//         let mut io_status: IO_STATUS_BLOCK = unsafe { core::mem::zeroed() };
        
//         let status = unsafe {
//             NtDeviceIoControlFile(
//                 self.handle.0,
//                 null_mut(),
//                 None,
//                 null_mut(),
//                 &mut io_status,
//                 FSCTL_ENUM_USN_DATA,
//                 &mut self.enum_data as *mut _ as _,
//                 size_of::<MFT_ENUM_DATA>() as u32,
//                 self.buffer.as_mut_ptr() as _,
//                 self.buffer.len() as u32,
//             )
//         };
        
//         if status < 0 {
//             return Err("FSCTL_ENUM_USN_DATA failed");
//         }
        
//         self.bytes_returned = io_status.Information as usize;
//         if self.bytes_returned <= 8 {
//             self.done = true;
//             return Ok(false);
//         }
        
//         self.next_start = unsafe { *(self.buffer.as_ptr() as *const u64) };
//         self.offset = 8;
//         Ok(true)
//     }
// }

// impl Iterator for UsnJournalIter {
//     type Item = UsnRecord;

//     fn next(&mut self) -> Option<Self::Item> {
//         loop {
//             if self.done {
//                 return None;
//             }
            
//             if self.offset >= self.bytes_returned {
//                 match self.refill_buffer() {
//                     Ok(true) => continue,
//                     Ok(false) => return None,
//                     Err(_) => {
//                         self.done = true;
//                         return None;
//                     }
//                 }
//             }
            
//             let record_ptr = unsafe { 
//                 self.buffer.as_ptr().add(self.offset) as *const USN_RECORD_V2 
//             };
//             let record = unsafe { &*record_ptr };
            
//             if record.MajorVersion == 2 || record.MajorVersion == 3 {
//                 let name_len = record.FileNameLength as usize / 2;
//                 let name_ptr = unsafe {
//                     self.buffer.as_ptr()
//                         .add(self.offset + record.FileNameOffset as usize)
//                         as *const u16
//                 };
//                 let file_name = unsafe {
//                     core::slice::from_raw_parts(name_ptr, name_len)
//                 };
                
//                 let result = UsnRecord {
//                     file_reference_number: record.FileReferenceNumber,
//                     parent_file_reference_number: record.ParentFileReferenceNumber,
//                     file_name: file_name.to_vec(),
//                     is_directory: record.FileAttributes & 0x10 != 0,
//                 };
                
//                 self.offset += record.RecordLength as usize;
//                 return Some(result);
//             }
            
//             self.offset += record.RecordLength as usize;
//         }
//     }
// }


fn get_file_size_by_frn(frn: u64) -> u64 {
    unsafe {
        let mut handle: HANDLE = null_mut();
        let mut io_status: IO_STATUS_BLOCK = core::mem::zeroed();
        
        let volume_path: Vec<u16> = r"\\.\C:".encode_utf16().collect();
        let mut path_uc = UNICODE_STRING {
            Length: (volume_path.len() * 2) as u16,
            MaximumLength: (volume_path.len() * 2 + 2) as u16,
            Buffer: volume_path.as_ptr() as *mut u16,
        };
        
        let mut object_attributes = OBJECT_ATTRIBUTES {
            Length: size_of::<OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: null_mut(),
            ObjectName: &mut path_uc,
            Attributes: OBJ_CASE_INSENSITIVE,
            SecurityDescriptor: null_mut(),
            SecurityQualityOfService: null_mut(),
        };
        
        let status = NtOpenFile(
            &mut handle,
            SYNCHRONIZE | FILE_READ_ATTRIBUTES,
            &mut object_attributes,
            &mut io_status,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            FILE_OPEN_BY_FILE_ID | FILE_OPEN_FOR_BACKUP_INTENT,
        );
        
        if status < 0 {
            return 0;
        }
        
        let mut standard_info: FILE_STANDARD_INFORMATION = core::mem::zeroed();
        let status = NtQueryInformationFile(
            handle,
            &mut io_status,
            &mut standard_info as *mut _ as _,
            size_of::<FILE_STANDARD_INFORMATION>() as u32,
            FileStandardInformation,
        );
        
        NtClose(handle);
        
        if status < 0 {
            0
        } else {
            *standard_info.EndOfFile.QuadPart() as u64
        }
    }
}

pub fn search_via_volume_winapi() {
    let volume_path: Vec<u16> = r"\\.\C:".encode_utf16().collect();

    let handle = match open_volume(&volume_path) {
        Ok(h) => h,
        Err(e) => {
            println!("Failed to open volume. Error: {}", e);
            return;
        }
    };
    
    let journal_data = match query_usn_journal(handle) {
        Ok(data) => data,
        Err(e) => {
            println!("Failed to query USN journal. Error: {}", e);
            unsafe { winapi::um::handleapi::CloseHandle(handle); }
            return;
        }
    };
    
    println!("USN Journal found! MaxUsn: {}", journal_data.MaxUsn);
    
    let dir_map = enumerate_directories(handle, journal_data.MaxUsn);
    println!("Found {} directories", dir_map.len());
    
    let mut file_count = 0;
    process_files(handle, journal_data.MaxUsn, &dir_map, |path, frn| {
        let size = get_file_size_by_frn(frn);
        let size_str = format_file_size(size);
        let path_str = String::from_utf16_lossy(path);
        println!("{} - {}", path_str, size_str);
        
        file_count += 1;
        if file_count % 10000 == 0 {
            println!("Processed {} files", file_count);
        }
    });
    
    unsafe { winapi::um::handleapi::CloseHandle(handle); }
    println!("Total files: {}", file_count);
}

// pub fn search_via_volume() {

//     // let volume = U16CStackString::<260>::from_str(r#"\??\C:"#).unwrap();
//     let volume = U16CStackString::<260>::from_str(r#"\Device\HarddiskVolume1"#).unwrap();

//     match UsnJournalIter::new(volume.as_slice()) {
//         Ok(iter) => {
//             let mut count = 0;
//             for record in iter {
//                 count += 1;
//                 if count % 10000 == 0 {
//                     let name = String::from_utf16_lossy(&record.file_name);
//                     println!("{}: {}", count, name);
//                 }
//             }
//             println!("Found {} records", count);
//         }
//         Err(err) => {
//             println!("Error: {}", err);
//         }
//     }
// }