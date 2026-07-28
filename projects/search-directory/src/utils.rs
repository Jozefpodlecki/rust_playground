use core::ptr::null_mut;
use core::mem::size_of;
use alloc::collections::btree_map::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use ntapi::ntioapi::{FILE_OPEN_BY_FILE_ID, FILE_OPEN_FOR_BACKUP_INTENT, FILE_STANDARD_INFORMATION, FILE_SYNCHRONOUS_IO_NONALERT, FileStandardInformation, IO_STATUS_BLOCK, NtDeviceIoControlFile, NtOpenFile};
use winapi::shared::ntdef::{HANDLE, OBJ_CASE_INSENSITIVE, OBJECT_ATTRIBUTES, UNICODE_STRING, LARGE_INTEGER};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::fileapi::{CreateFileW, OPEN_EXISTING};
use winapi::um::ioapiset::DeviceIoControl;
use winapi::um::winbase::FILE_FLAG_BACKUP_SEMANTICS;
use winapi::um::winnt::{FILE_ATTRIBUTE_NORMAL, FILE_READ_ATTRIBUTES, FILE_READ_DATA, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE, SYNCHRONIZE};
use winapi::um::winioctl::{FSCTL_ENUM_USN_DATA, FSCTL_QUERY_USN_JOURNAL};
use toolkit::syscalls::{NtClose, NtQueryInformationFile};
use toolkit::{U16CStackString, println};

use crate::types::*;

pub fn open_volume(volume_path: &[u16]) -> Result<HANDLE, u32> {
    unsafe {
        let handle = CreateFileW(
            volume_path.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            core::ptr::null_mut(),
            OPEN_EXISTING,
            0,
            core::ptr::null_mut(),
        );
        
        if handle.is_null() {
            return Err(GetLastError());
        }
        Ok(handle)
    }
}

pub fn query_usn_journal(handle: HANDLE) -> Result<USN_JOURNAL_DATA, u32> {
    unsafe {
        let mut journal_data: USN_JOURNAL_DATA = core::mem::zeroed();
        let mut bytes_returned = 0;
        
        let result = DeviceIoControl(
            handle,
            FSCTL_QUERY_USN_JOURNAL,
            core::ptr::null_mut(),
            0,
            &mut journal_data as *mut _ as _,
            size_of::<USN_JOURNAL_DATA>() as u32,
            &mut bytes_returned,
            core::ptr::null_mut(),
        );
        
        if result == 0 {
            return Err(GetLastError());
        }
        Ok(journal_data)
    }
}


pub fn enumerate_directories(handle: HANDLE, max_usn: i64) -> BTreeMap<u64, (Vec<u16>, u64)> {
    let mut dir_map = BTreeMap::new();
    let mut enum_data = MFT_ENUM_DATA {
        StartFileReferenceNumber: 0,
        LowUsn: 0,
        HighUsn: max_usn,
    };
    let mut buffer = AlignedBuffer::<65536>::new();
    let mut total_dirs = 0;
    let mut loop_count = 0;
    
    loop {
        loop_count += 1;
        let mut bytes_returned = 0;
        unsafe {
            let result = DeviceIoControl(
                handle,
                FSCTL_ENUM_USN_DATA,
                &mut enum_data as *mut _ as _,
                size_of::<MFT_ENUM_DATA>() as u32,
                buffer.as_mut_ptr() as _,
                buffer.len() as u32,
                &mut bytes_returned,
                core::ptr::null_mut(),
            );
            
            if result == 0 {
                let error = GetLastError();
                println!("DeviceIoControl failed with error: {}", error);
                break;
            }
            
            if bytes_returned <= 8 {
                println!("bytes_returned <= 8, breaking");
                break;
            }
        }
        
        let next_start = unsafe { *(buffer.as_ptr() as *const u64) };
        
        let mut offset = 8usize;
        let mut records_in_buffer = 0;
        
        while offset < bytes_returned as usize {
            let record_ptr = unsafe { buffer.as_ptr().add(offset) as *const USN_RECORD_V2 };
            let record = unsafe { &*record_ptr };

            if record.MajorVersion == 4 {
                unimplemented!("test");
            }
            
            if record.MajorVersion == 2 || record.MajorVersion == 3 {
                let is_dir = record.FileAttributes & 0x10 != 0;
                if is_dir {
                    let name = record.file_name(buffer.as_ref(), offset);
                    let name_str = String::from_utf16_lossy(name);
                    // println!("  Directory: {} (FRN: {}, Parent: {})", name_str, record.FileReferenceNumber, record.ParentFileReferenceNumber);
                    dir_map.insert(record.FileReferenceNumber, (name.to_vec(), record.ParentFileReferenceNumber));
                    total_dirs += 1;
                }
            }
            
            offset += record.RecordLength as usize;
            records_in_buffer += 1;
        }
        

        enum_data.StartFileReferenceNumber = next_start;
        if next_start == 0 {
            println!("next_start is 0, breaking");
            break;
        }
    }
    
    println!("Total directories found: {}", total_dirs);
    dir_map
}

pub fn process_files<F>(handle: HANDLE, max_usn: i64, dir_map: &BTreeMap<u64, (Vec<u16>, u64)>, mut callback: F)
where
    F: FnMut(&[u16], u64),
{
    let mut enum_data = MFT_ENUM_DATA {
        StartFileReferenceNumber: 0,
        LowUsn: 0,
        HighUsn: max_usn,
    };
    let mut buffer = [0u8; 65536];
    let root_frn = 5;
    
    loop {
        let mut bytes_returned = 0;
        unsafe {
            let result = DeviceIoControl(
                handle,
                FSCTL_ENUM_USN_DATA,
                &mut enum_data as *mut _ as _,
                size_of::<MFT_ENUM_DATA>() as u32,
                buffer.as_mut_ptr() as _,
                buffer.len() as u32,
                &mut bytes_returned,
                core::ptr::null_mut(),
            );
            
            if result == 0 || bytes_returned <= 8 {
                break;
            }
        }
        
        let next_start = unsafe { *(buffer.as_ptr() as *const u64) };
        let mut offset = 8usize;
        
        while offset < bytes_returned as usize {
            let record_ptr = unsafe { buffer.as_ptr().add(offset) as *const USN_RECORD_V2 };
            let record = unsafe { &*record_ptr };
            
            if record.MajorVersion == 2 || record.MajorVersion == 3 {
                let is_dir = record.FileAttributes & 0x10 != 0;
                if !is_dir {
                    let name = record.file_name(&buffer, offset);
                    let path = build_full_path(
                        record.FileReferenceNumber,
                        record.ParentFileReferenceNumber,
                        name,
                        dir_map,
                        root_frn,
                    );
                    callback(&path, record.FileReferenceNumber);
                }
            }
            
            offset += record.RecordLength as usize;
        }
        
        enum_data.StartFileReferenceNumber = next_start;
        if next_start == 0 {
            break;
        }
    }
}

pub fn build_full_path(
    frn: u64,
    parent_frn: u64,
    name: &[u16],
    dir_map: &BTreeMap<u64, (Vec<u16>, u64)>,
    root_frn: u64,
) -> Vec<u16> {
    let mut path = Vec::new();
    let mut current = parent_frn;
    
    while current != root_frn && current != 0 {
        if let Some((dir_name, parent)) = dir_map.get(&current) {
            path.extend_from_slice(dir_name);
            path.push(b'\\' as u16);
            current = *parent;
        } else {
            break;
        }
    }
    
    path.push(b'\\' as u16);
    path.push(b'C' as u16);
    path.push(b':' as u16);
    path.push(b'\\' as u16);
    path.reverse();
    path.extend_from_slice(name);
    path
}

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

pub fn format_file_size(size: u64) -> String {
    if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.2} KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.2} MB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}