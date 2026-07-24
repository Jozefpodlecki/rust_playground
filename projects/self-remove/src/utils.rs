
// use core::{mem, ptr::null_mut};

// use ntapi::ntpsapi::{NtQueryInformationThread, ThreadQuerySetWin32StartAddress};
// use toolkit::ProcessMemoryReader ;
// use winapi::{shared::{minwindef::ULONG, ntdef::{HANDLE, NTSTATUS, PVOID}}, um::winnt::{IMAGE_DATA_DIRECTORY, IMAGE_DIRECTORY_ENTRY_EXPORT, IMAGE_DOS_HEADER, IMAGE_EXPORT_DIRECTORY, IMAGE_NT_HEADERS64}};

// use crate::{injector::*, shellcode::Shellcode};

// use alloc::vec::Vec;
// use ntapi::{ntioapi::{FILE_BASIC_INFORMATION, FILE_NON_DIRECTORY_FILE, FILE_STANDARD_INFORMATION, FILE_SYNCHRONOUS_IO_NONALERT, FileBasicInformation, FileDispositionInformation, FileStandardInformation, IO_STATUS_BLOCK}, ntobapi::NtClose, };
// use winapi::{ctypes::c_void, shared::ntdef::{ OBJ_CASE_INSENSITIVE, OBJECT_ATTRIBUTES}, um::{fileapi::FILE_DISPOSITION_INFO, winnt::{DELETE, FILE_ATTRIBUTE_READONLY, FILE_READ_DATA, FILE_SHARE_DELETE, FILE_SHARE_READ, SYNCHRONIZE}}};
// use toolkit::{syscalls::{NtOpenFile, NtQueryInformationFile, NtSetInformationFile}, *};


// pub fn is_delete_pending(handle: HANDLE) -> bool {
//     unsafe {
//         let mut info: FILE_STANDARD_INFORMATION = core::mem::zeroed();
//         let mut io_status_block: IO_STATUS_BLOCK = core::mem::zeroed();

//         let status = NtQueryInformationFile(
//             handle,
//             &mut io_status_block,
//             &mut info as *mut _ as *mut _,
//             core::mem::size_of::<FILE_STANDARD_INFORMATION>() as u32,
//             FileStandardInformation,
//         );

//         if status != 0 {
//             return false;
//         }

//         info.DeletePending != 0
//     }
// }

// pub fn diagnose_delete_failure(handle: HANDLE) {
//     let pending = is_delete_pending(handle);
//     println!("DeletePending: {}", pending);

//     let mut basic_info: FILE_BASIC_INFORMATION = unsafe { core::mem::zeroed() };
//     let mut io_status_block: IO_STATUS_BLOCK = unsafe { core::mem::zeroed() };
//     let status = NtQueryInformationFile(
//         handle,
//         &mut io_status_block,
//         &mut basic_info as *mut _ as *mut _,
//         core::mem::size_of::<FILE_BASIC_INFORMATION>() as u32,
//         FileBasicInformation,
//     );
//     println!("diagnose_delete_failure NtQueryInformationFile 0x{status:X}");
//     if status == 0 {
//         let is_readonly = (basic_info.FileAttributes & FILE_ATTRIBUTE_READONLY) != 0;
//         println!("READONLY attribute: {}", is_readonly);
//     }
// }

// pub fn set_information_file() {
//     let mut file_path = U16CStackString::<150>::from_str(
//         r#"\??\C:\repos\rust_playground\projects\self-remove\target\debug\self-remove.exe"#
//         // r#"\??\C:\repos\rust_playground\projects\self-remove\target\debug\self_remove.pdb"#
//     ).unwrap();

//     let mut object_attributes = OBJECT_ATTRIBUTES {
//         Length: size_of::<OBJECT_ATTRIBUTES>() as u32,
//         RootDirectory: null_mut(),
//         ObjectName: &mut file_path.to_unicode_string(),
//         Attributes: OBJ_CASE_INSENSITIVE,
//         SecurityDescriptor: null_mut(),
//         SecurityQualityOfService: null_mut(),
//     };
    
//     let mut handle: HANDLE = null_mut();
//     let mut status_block: IO_STATUS_BLOCK = unsafe { core::mem::zeroed() };
//     let access = DELETE | FILE_READ_DATA | SYNCHRONIZE;
//     let share = FILE_SHARE_READ | FILE_SHARE_DELETE;
//     let create_options = FILE_NON_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT;

//     let status = unsafe {
//         NtOpenFile(
//             &mut handle,
//             access,
//             &mut object_attributes,
//             &mut status_block,
//             share,
//             create_options,
//         )
//     };

//     println!("NtOpenFile 0x{status:X} {handle:p}");
//     diagnose_delete_failure(handle);
    
//     let mut info = FILE_DISPOSITION_INFO {
//         DeleteFile: 1,
//     };
//     let mut io_status_block: IO_STATUS_BLOCK = unsafe { core::mem::zeroed() };
    
//     // pub const STATUS_ACCESS_DENIED: NTSTATUS = 0xC0000022;
//     let status = NtSetInformationFile(
//         handle,
//         &mut io_status_block,
//         &mut info as *mut _ as *mut _,
//         core::mem::size_of::<FILE_DISPOSITION_INFO>() as u32,
//         FileDispositionInformation,
//     );

//     println!("NtSetInformationFile 0x{status:X}");
    
//     unsafe { NtClose(handle) };
// }

// pub fn get_thread_start_address(thread_handle: HANDLE) -> Result<*mut winapi::ctypes::c_void, NTSTATUS> {
//     let mut start_address: PVOID = core::ptr::null_mut();
//     let status = unsafe {
//         NtQueryInformationThread(
//             thread_handle,
//             ThreadQuerySetWin32StartAddress,
//             &mut start_address as *mut _ as PVOID,
//             core::mem::size_of::<PVOID>() as ULONG,
//             core::ptr::null_mut(),
//         )
//     };
//     // println!("0x{:X}", status);
//     if status == 0 {
//         Ok(start_address)
//     } else {
//         Err(status)
//     }
// }

// pub fn get_export(
//     handle: *mut winapi::ctypes::c_void,
//     base: *const winapi::ctypes::c_void,
//     name: &str,
// ) -> Result<*mut winapi::ctypes::c_void, NTSTATUS> {
//     unsafe {
//         let dos_header: IMAGE_DOS_HEADER = ProcessMemoryReader::read_remote(handle, base as _)?;
//         let nt_headers_addr: *mut winapi::ctypes::c_void = (base as usize + dos_header.e_lfanew as usize) as *mut _;
//         let nt_headers: IMAGE_NT_HEADERS64 = ProcessMemoryReader::read_remote(handle, nt_headers_addr as _)?;

//         let export_dir: IMAGE_DATA_DIRECTORY = nt_headers.OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_EXPORT as usize];
//         let export_addr: *mut winapi::ctypes::c_void = (base as usize + export_dir.VirtualAddress as usize) as *mut _;
//         let export_dir_struct: IMAGE_EXPORT_DIRECTORY = ProcessMemoryReader::read_remote(handle, export_addr as _)?;

//         let names_addr = (base as usize + export_dir_struct.AddressOfNames as usize) as *const winapi::ctypes::c_void;
//         let ordinals_addr = (base as usize + export_dir_struct.AddressOfNameOrdinals as usize) as *const winapi::ctypes::c_void;
//         let functions_addr = (base as usize + export_dir_struct.AddressOfFunctions as usize) as *const winapi::ctypes::c_void;

//         for i in 0..export_dir_struct.NumberOfNames {
//             let name_rva_addr = (names_addr as usize + (i as usize * 4)) as *const winapi::ctypes::c_void;
//             let name_rva: u32 = ProcessMemoryReader::read_remote(handle, name_rva_addr as _)?;
            
//             let name_ptr = (base as usize + name_rva as usize) as *const winapi::ctypes::c_void;
//             let mut name_bytes = [0u8; 256];
            
//             for j in 0..255 {
//                 let byte_addr = (name_ptr as usize + j) as *const winapi::ctypes::c_void;
//                 let byte: u8 = ProcessMemoryReader::read_remote(handle, byte_addr as _)?;
//                 name_bytes[j] = byte;
//                 if byte == 0 {
//                     break;
//                 }
//             }
            
//             let c_str = core::ffi::CStr::from_bytes_until_nul(&name_bytes);
//             if let Ok(c_str) = c_str {
//                 if let Ok(export_name) = c_str.to_str() {
//                     if export_name == name {
//                         let ordinal_addr = (ordinals_addr as usize + (i as usize * 2)) as *const winapi::ctypes::c_void;
//                         let ordinal: u16 = ProcessMemoryReader::read_remote(handle, ordinal_addr as _)?;
                        
//                         let function_rva_addr = (functions_addr as usize + (ordinal as usize * 4)) as *const winapi::ctypes::c_void;
//                         let function_rva: u32 = ProcessMemoryReader::read_remote(handle, function_rva_addr as _)?;
                        
//                         return Ok((base as usize + function_rva as usize) as *mut winapi::ctypes::c_void);
//                     }
//                 }
//             }
//         }

//         Err(0)
//     }
// }
