use core::{mem, ptr, slice};

use alloc::string::String;
use ntapi::{ntapi_base::CLIENT_ID, ntioapi::{FILE_BASIC_INFORMATION, FileBasicInformation, FileFsVolumeInformation, FileNameInformation, IO_STATUS_BLOCK, NtQueryInformationFile, NtQueryVolumeInformationFile}, ntobapi::{DIRECTORY_QUERY, DUPLICATE_SAME_ACCESS, NtDuplicateObject, NtOpenDirectoryObject, NtOpenSymbolicLinkObject, NtQueryDirectoryObject, NtQueryObject, NtQuerySymbolicLinkObject, OBJ_INHERIT, OBJECT_DIRECTORY_INFORMATION, OBJECT_NAME_INFORMATION, ObjectNameInformation, SYMBOLIC_LINK_QUERY}, ntpsapi::{NtCurrentProcess, NtOpenProcess}};
use toolkit::{ProcessMemoryReader, STD_OUTPUT_HANDLE, println};
use winapi::{shared::{minwindef::FALSE, ntdef::{HANDLE, OBJECT_ATTRIBUTES, UNICODE_STRING}, ntstatus::{STATUS_MORE_ENTRIES, STATUS_SUCCESS}}, um::{fileapi::{GetFileType, GetFinalPathNameByHandleW}, handleapi::{CloseHandle, DuplicateHandle}, processthreadsapi::{GetCurrentProcess, OpenProcess}, winbase::{FILE_TYPE_CHAR, FILE_TYPE_DISK, FILE_TYPE_PIPE, FILE_TYPE_UNKNOWN, STD_ERROR_HANDLE, STD_INPUT_HANDLE, VOLUME_NAME_DOS}, winnt::{FILE_ALL_ACCESS, FILE_READ_ATTRIBUTES, PROCESS_ALL_ACCESS, PROCESS_DUP_HANDLE}}};

use crate::{handle::{HandleInfo, SystemHandleIterator}, object_type::ObjectTypeIterator};

// ---------- Configuration ----------
const DEBUG: bool = true; // set to false to disable all debug output

macro_rules! debug_println {
    ($($arg:tt)*) => {
        if DEBUG {
            println!($($arg)*);
        }
    };
}

#[repr(align(8))]
struct AlignedBuffer<const N: usize>([u8; N]);

pub struct FileName {
    data: [u8; 256],
    len: usize,
}

impl FileName {
    pub fn from_str(s: &str) -> Self {
        let mut data = [0u8; 256];
        let bytes = s.as_bytes();
        let len = bytes.len().min(data.len());
        data[..len].copy_from_slice(&bytes[..len]);
        Self { data, len }
    }

    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.data[..self.len]) }
    }
}

pub fn duplicate_handle(handle_info: &HandleInfo) -> Option<HANDLE> {
    unsafe {
        let mut target_process = HANDLE::default();
        let mut client_id = CLIENT_ID {
            UniqueProcess: handle_info.process_id() as *mut _,
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

        let status = NtOpenProcess(
            &mut target_process,
            PROCESS_ALL_ACCESS,
            &mut object_attributes,
            &mut client_id,
        );
        if status != 0 {
            debug_println!("NtOpenProcess failed: 0x{:X}", status);
            return None;
        }

        let current_process = NtCurrentProcess;
        let mut duplicated_handle = core::ptr::null_mut();

        let status = NtDuplicateObject(
            target_process,
            handle_info.handle_value() as *mut _,
            current_process,
            &mut duplicated_handle,
            FILE_ALL_ACCESS,
            OBJ_INHERIT,
            DUPLICATE_SAME_ACCESS,
        );
        if status != 0 {
            debug_println!("NtDuplicateObject failed: 0x{:X}", status);
            return None;
        }

        Some(duplicated_handle)
    }
}

fn wide_to_utf8(wide: &[u16]) -> String {
    let mut utf8 = String::new();
    for c in char::decode_utf16(wide.iter().cloned()) {
        if let Ok(ch) = c {
            utf8.push(ch);
        }
    }
    utf8
}

fn unicode_to_string(unicode: &UNICODE_STRING) -> String {
    if unicode.Length == 0 || unicode.Buffer.is_null() {
        return String::new();
    }
    let char_count = (unicode.Length / 2) as usize;
    unsafe {
        let wide_slice = slice::from_raw_parts(unicode.Buffer, char_count);
        wide_to_utf8(wide_slice)
    }
}

pub fn is_directory_handle(handle: HANDLE) -> bool {
    unsafe {
        let mut buffer = [0u8; 40];
        let mut io_status_block: IO_STATUS_BLOCK = core::mem::zeroed();
        let status = NtQueryInformationFile(
            handle,
            &mut io_status_block,
            buffer.as_mut_ptr() as *mut _,
            buffer.len() as u32,
            FileBasicInformation,
        );
        if status == STATUS_SUCCESS {
            let info = &*(buffer.as_ptr() as *const FILE_BASIC_INFORMATION);
            return (info.FileAttributes & 0x10) != 0; // FILE_ATTRIBUTE_DIRECTORY
        }
        println!("NtQueryInformationFile 0x{:X}", status);
        false
    }
}

fn get_relative_path(handle: HANDLE) -> Option<String> {
    unsafe {
        let mut aligned_buffer = AlignedBuffer([0u8; 1024]);
        let buffer_ptr = aligned_buffer.0.as_mut_ptr();
        let mut io_status_block: IO_STATUS_BLOCK = core::mem::zeroed();

        let status = NtQueryInformationFile(
            handle,
            &mut io_status_block,
            buffer_ptr as *mut _,
            aligned_buffer.0.len() as u32,
            FileNameInformation,
        );
        if status != STATUS_SUCCESS {
            debug_println!("NtQueryInformationFile failed: 0x{:X}", status);
            return None;
        }
        debug_println!("NtQueryInformationFile succeeded");

        let len_bytes = *(buffer_ptr as *const u32);
        let char_count = (len_bytes / 2) as usize;
        let wide_ptr = buffer_ptr.add(4) as *const u16;
        let wide_slice = slice::from_raw_parts(wide_ptr, char_count);
        let path = wide_to_utf8(wide_slice);
        debug_println!("Relative path: {}", path);
        Some(path)
    }
}

pub fn get_file_type(handle: HANDLE) -> u32 {
    unsafe {
        let std_handles = [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE];
        let current_handle = handle as isize;
        
        // for &std_handle in &std_handles {
        //     let std_h = GetStdHandle(std_handle);
        //     if !std_h.is_null() && std_h as isize == current_handle {
        //         return FILE_TYPE_CHAR;
        //     }
        // }

        // if current_handle < 0 {
        //     return FILE_TYPE_CHAR;
        // }
        
        let mut buffer = [0u8; 256];
        let mut io_status_block: IO_STATUS_BLOCK = core::mem::zeroed();
        
        let status = NtQueryVolumeInformationFile(
            handle,
            &mut io_status_block,
            buffer.as_mut_ptr() as *mut _,
            buffer.len() as u32,
            FileFsVolumeInformation,
        );
        
        if status == STATUS_SUCCESS {
            // It's a volume/device - FILE_TYPE_CHAR or FILE_TYPE_DISK
            // Check if it's a character device (like console, serial, etc.)
            // For simplicity, we assume it's a disk file
            return FILE_TYPE_DISK;
        }

        // If NtQueryVolumeInformationFile fails, it might be a pipe or unknown
        // Try NtQueryInformationFile to check if it's a pipe
        let mut pipe_info = [0u8; 16];
        const FilePipeInformation: u32 = 23;
        
        let status2 = NtQueryInformationFile(
            handle,
            &mut io_status_block,
            pipe_info.as_mut_ptr() as *mut _,
            pipe_info.len() as u32,
            FilePipeInformation,
        );
        
        if status2 == STATUS_SUCCESS {
            return FILE_TYPE_PIPE;
        }

        FILE_TYPE_UNKNOWN
    }
}

pub fn get_nt_path(handle: HANDLE) -> Option<String> {
    unsafe {
        let mut aligned_buffer = AlignedBuffer([0u8; 1024]);
        let buffer_ptr = aligned_buffer.0.as_mut_ptr();
        let mut return_len = 0;
        println!("0x{:X}", GetFileType(handle));
        let status = NtQueryObject(
            handle,
            ObjectNameInformation,
            buffer_ptr as *mut _,
            aligned_buffer.0.len() as u32,
            &mut return_len,
        );

        if status != STATUS_SUCCESS {
            debug_println!("NtQueryObject failed: 0x{:X}", status);
            return None;
        }
        debug_println!("NtQueryObject succeeded, return_len={}", return_len);

        let obj_name = &*(buffer_ptr as *const OBJECT_NAME_INFORMATION);
        let name_unicode = &obj_name.Name;
        let name_len = name_unicode.Length as usize;
        let name_buffer = name_unicode.Buffer;

        if name_len == 0 || name_buffer.is_null() {
            debug_println!("Empty NT name");
            return None;
        }

        // Bounds check (optional)
        let buf_start = buffer_ptr as usize;
        let str_addr = name_buffer as usize;
        let buf_end = buf_start + aligned_buffer.0.len();
        if str_addr < buf_start || str_addr + name_len > buf_end {
            debug_println!("NT name pointer out of bounds");
            return None;
        }

        let wide_slice = slice::from_raw_parts(name_buffer, name_len / 2);
        let path = wide_to_utf8(wide_slice);
        debug_println!("NT path: {}", path);
        Some(path)
    }
}

// ---------- Extract device from NT path ----------
fn extract_device(nt_path: &str) -> Option<&str> {
    if nt_path.len() < 8 {
        debug_println!("NT path too short");
        return None;
    }
    let device_end = nt_path[8..].find('\\').map(|i| i + 8).unwrap_or(nt_path.len());
    let device = &nt_path[..device_end];
    debug_println!("Extracted device: {}", device);
    Some(device)
}

// ---------- Open DOS device directory ----------
fn open_dos_device_directory() -> Option<HANDLE> {
    unsafe {
        // Try \GLOBAL?? first (the real DOS device directory)
        let mut dir_handle = HANDLE::default();
        let mut name_buf: [u16; 10] = [
            0x5C, 0x47, 0x4C, 0x4F, 0x42, 0x41, 0x4C, 0x3F, 0x3F, 0x00,
        ]; // L"\\GLOBAL??"
        let mut name = UNICODE_STRING {
            Length: 18,
            MaximumLength: 20,
            Buffer: name_buf.as_mut_ptr(),
        };
        let mut obj_attr = OBJECT_ATTRIBUTES {
            Length: mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: ptr::null_mut(),
            ObjectName: &mut name,
            Attributes: 0x40,
            SecurityDescriptor: ptr::null_mut(),
            SecurityQualityOfService: ptr::null_mut(),
        };
        let status = NtOpenDirectoryObject(
            &mut dir_handle,
            DIRECTORY_QUERY,
            &mut obj_attr,
        );
        debug_println!("open_dos_device_directory(\\GLOBAL??) status = 0x{:X}", status);
        if status == STATUS_SUCCESS {
            return Some(dir_handle);
        }

        // Fallback to \?? (older systems)
        let mut fallback_buf: [u16; 4] = [0x5C, 0x3F, 0x3F, 0x00]; // L"\\??"
        let mut fallback_name = UNICODE_STRING {
            Length: 6,
            MaximumLength: 8,
            Buffer: fallback_buf.as_mut_ptr(),
        };
        obj_attr.ObjectName = &mut fallback_name;
        let status2 = NtOpenDirectoryObject(
            &mut dir_handle,
            DIRECTORY_QUERY,
            &mut obj_attr,
        );
        debug_println!("open_dos_device_directory(\\??) status = 0x{:X}", status2);
        if status2 == STATUS_SUCCESS {
            Some(dir_handle)
        } else {
            None
        }
    }
}

// ---------- Enumerate symbolic links in DOS device directory ----------
fn enumerate_dos_links<F>(dir_handle: HANDLE, mut callback: F) -> Option<char>
where
    F: FnMut(&str, &str) -> Option<char>,
{
    unsafe {
        let mut aligned_buffer = AlignedBuffer([0u8; 4096]);
        let mut context = 0u32;
        let mut first: u8 = 1;
        let mut iteration = 0;

        loop {
            iteration += 1;
            let mut return_len = 0;
            let status = NtQueryDirectoryObject(
                dir_handle,
                aligned_buffer.0.as_mut_ptr() as *mut _,
                aligned_buffer.0.len() as u32,
                0,
                first,
                &mut context,
                &mut return_len,
            );
            debug_println!(
                "NtQueryDirectoryObject iteration {}: status=0x{:X}, return_len={}",
                iteration, status, return_len
            );
            if status != STATUS_SUCCESS && status != STATUS_MORE_ENTRIES {
                debug_println!("Breaking due to error status");
                break;
            }
            first = 0;

            let mut offset = 0;
            while offset < return_len as usize {
                let entry = aligned_buffer.0.as_ptr().add(offset) as *const OBJECT_DIRECTORY_INFORMATION;
                let entry_ref = &*entry;
                let name_len = entry_ref.Name.Length;
                if name_len == 0 {
                    break;
                }

                let type_name = unicode_to_string(&entry_ref.TypeName);
                let link_name = unicode_to_string(&entry_ref.Name);
                debug_println!("Entry: name='{}', type='{}'", link_name, type_name);

                if type_name == "SymbolicLink" {
                    // Open link and query target
                    let mut link_name_unicode = entry_ref.Name;
                    let mut link_handle = HANDLE::default();
                    let mut link_attr = OBJECT_ATTRIBUTES {
                        Length: mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
                        RootDirectory: dir_handle,
                        ObjectName: &mut link_name_unicode,
                        Attributes: 0x40,
                        SecurityDescriptor: ptr::null_mut(),
                        SecurityQualityOfService: ptr::null_mut(),
                    };
                    let stat = NtOpenSymbolicLinkObject(
                        &mut link_handle,
                        SYMBOLIC_LINK_QUERY,
                        &mut link_attr,
                    );
                    if stat == STATUS_SUCCESS {
                        let mut target_buf = [0u8; 512];
                        let mut target_unicode = UNICODE_STRING {
                            Length: 0,
                            MaximumLength: target_buf.len() as u16,
                            Buffer: target_buf.as_mut_ptr() as *mut u16,
                        };
                        let mut return_len2 = 0u32;
                        let stat2 = NtQuerySymbolicLinkObject(
                            link_handle,
                            &mut target_unicode,
                            &mut return_len2,
                        );
                        if stat2 == STATUS_SUCCESS {
                            let target = unicode_to_string(&target_unicode);
                            debug_println!("  link '{}' -> target '{}'", link_name, target);
                            if let Some(ch) = callback(&link_name, &target) {
                                CloseHandle(link_handle);
                                return Some(ch);
                            }
                        } else {
                            debug_println!("  NtQuerySymbolicLinkObject failed: 0x{:X}", stat2);
                        }
                        CloseHandle(link_handle);
                    } else {
                        debug_println!("  NtOpenSymbolicLinkObject failed: 0x{:X}", stat);
                    }
                }
                offset += mem::size_of::<OBJECT_DIRECTORY_INFORMATION>();
            }
            if status != STATUS_MORE_ENTRIES {
                debug_println!("No more entries");
                break;
            }
            debug_println!("Continuing to next iteration...");
        }
        None
    }
}

fn map_device_to_dos(device: &str) -> Option<char> {
    debug_println!("map_device_to_dos: looking for '{}'", device);

    let dir_handle = match open_dos_device_directory() {
        Some(h) => h,
        None => {
            debug_println!("Failed to open DOS device directory");
            return None;
        }
    };

    let result = enumerate_dos_links(dir_handle, |link_name, target| {

        if !link_name.ends_with(':') {
            return None;
        }
        let target_trimmed = target.trim_end_matches('\\');
        let device_trimmed = device.trim_end_matches('\\');
        if target_trimmed.eq_ignore_ascii_case(device_trimmed) {
            // Return the first character (the drive letter)
            link_name.chars().next()
        } else {
            None
        }
    });

    unsafe { CloseHandle(dir_handle); }

    if result.is_none() {
        debug_println!("No drive-letter mapping found for device '{}'", device);
    }
    result
}

pub fn get_full_path_manual(handle: HANDLE) -> Option<FileName> {
    let nt_path = get_nt_path(handle)?;

    if nt_path.starts_with("\\Device\\ConDrv") || 
       nt_path.starts_with("\\Device\\Null") ||
       nt_path.starts_with("\\Device\\NamedPipe") {
        return Some(FileName::from_str(&nt_path));
    }

    let rel_path = get_relative_path(handle)?;
    let device = extract_device(&nt_path)?;
    let drive_letter = map_device_to_dos(device).unwrap_or('C');

    let rel_trimmed = rel_path.trim_start_matches('\\');
    let full = alloc::format!("{}:\\{}", drive_letter, rel_trimmed);
    debug_println!("Final path: {}", full);

    let mut final_buf = [0u8; 256];
    let bytes = full.as_bytes();
    let len = bytes.len().min(final_buf.len());
    final_buf[..len].copy_from_slice(&bytes[..len]);

    Some(FileName { data: final_buf, len })
}

pub fn get_full_path_from_handle(handle: *mut winapi::ctypes::c_void) -> Option<FileName> {
    unsafe {
        let mut wide_buf = [0u16; 1024];
        let len = GetFinalPathNameByHandleW(
            handle,
            wide_buf.as_mut_ptr(),
            wide_buf.len() as u32,
            VOLUME_NAME_DOS,
        );
        if len == 0 {
            return None;
        }
        let char_count = len as usize;
        if char_count > wide_buf.len() {
            return None;
        }
        let wide_slice = &wide_buf[..char_count];
        let mut utf8_buf = [0u8; 256];
        let mut idx = 0;
        for c in char::decode_utf16(wide_slice.iter().cloned()) {
            if let Ok(ch) = c {
                let mut bytes = [0u8; 4];
                let encoded = ch.encode_utf8(&mut bytes);
                let bytes_slice = encoded.as_bytes();
                if idx + bytes_slice.len() > utf8_buf.len() {
                    break;
                }
                for &b in bytes_slice {
                    utf8_buf[idx] = b;
                    idx += 1;
                }
            }
        }
        Some(FileName { data: utf8_buf, len: idx })
    }
}
