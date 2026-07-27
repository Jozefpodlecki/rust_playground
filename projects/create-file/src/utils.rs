
use core::ptr::null_mut;

use ntapi::{ntioapi::{FILE_END_OF_FILE_INFORMATION, FILE_POSITION_INFORMATION, FILE_STANDARD_INFORMATION, FILE_VALID_DATA_LENGTH_INFORMATION, FileEndOfFileInformation, FilePositionInformation, FileStandardInformation, FileValidDataLengthInformation, IO_STATUS_BLOCK, NtFsControlFile}, ntpsapi::{NtCurrentProcess, NtTerminateProcess}};
use toolkit::{File, ProcessEnvironmentBlock, ProcessSpawner, U16CStackString, canonicalize, println, syscalls::{NtQueryInformationFile, NtSetInformationFile}};
use winapi::{shared::{ntdef::HANDLE, ntstatus::STATUS_SUCCESS}, um::{errhandlingapi::GetLastError, fileapi::{SetEndOfFile, SetFilePointerEx, SetFileValidData}, handleapi::CloseHandle, processthreadsapi::{GetCurrentProcess, OpenProcessToken}, securitybaseapi::AdjustTokenPrivileges, winbase::LookupPrivilegeValueW, winioctl::{FSCTL_SET_SPARSE, FSCTL_SET_ZERO_DATA}, winnt::{LARGE_INTEGER, LUID, SE_MANAGE_VOLUME_NAME, SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY}}};

pub fn enable_manage_volume_privilege() -> bool {
    unsafe {
        let mut token: HANDLE = null_mut();

        if OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token,
        ) == 0 {
            let error = GetLastError();
            println!("OpenProcessToken failed with error: {}", error);
            return false;
        }

        let mut luid: LUID = core::mem::zeroed();
        let privilege_name = U16CStackString::<100>::from_str(SE_MANAGE_VOLUME_NAME).unwrap();
        
        if LookupPrivilegeValueW(null_mut(), privilege_name.as_ptr(), &mut luid) == 0 {
            let error = GetLastError();
            println!("LookupPrivilegeValueW failed with error: {}", error);
            CloseHandle(token);
            return false;
        }

        let mut tp: TOKEN_PRIVILEGES = core::mem::zeroed();
        tp.PrivilegeCount = 1;
        tp.Privileges[0].Luid = luid;
        tp.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;

        if AdjustTokenPrivileges(token, 0, &mut tp, 0, null_mut(), null_mut()) == 0 {
            let error = GetLastError();
            println!("AdjustTokenPrivileges failed with error: {}", error);
            CloseHandle(token);
            return false;
        }

        // Check if the privilege was actually enabled
        let result = GetLastError();
        if result != 0 {
            println!("AdjustTokenPrivileges didn't enable privilege, error: {}", result);
            CloseHandle(token);
            return false;
        }

        CloseHandle(token);
        true
    }
}

pub fn nt_mark_sparse(handle: HANDLE) -> bool {
    unsafe {
        let mut io_status: IO_STATUS_BLOCK = core::mem::zeroed();
        
        // FSCTL_SET_SPARSE with 1 = enable sparse
        let enable = 1u32;
        let status = NtFsControlFile(
            handle,
            null_mut(),
            None,
            null_mut(),
            &mut io_status,
            FSCTL_SET_SPARSE,
            &enable as *const _ as *mut _,
            core::mem::size_of::<u32>() as u32,
            null_mut(),
            0,
        );
        
        status == STATUS_SUCCESS
    }
}

#[repr(C)]
pub struct FILE_ZERO_DATA_INFORMATION {
    pub FileOffset: LARGE_INTEGER,
    pub BeyondFinalZero: LARGE_INTEGER,
}

pub fn nt_zero_range(handle: HANDLE, offset: u64, length: u64) -> bool {
    unsafe {
        let mut io_status: IO_STATUS_BLOCK = core::mem::zeroed();
        
        let mut zero_info: FILE_ZERO_DATA_INFORMATION = core::mem::zeroed();
        *zero_info.FileOffset.QuadPart_mut() = offset as i64;
        *zero_info.BeyondFinalZero.QuadPart_mut() = (offset + length) as i64;
        
        let status = NtFsControlFile(
            handle,
            null_mut(),
            None,
            null_mut(),
            &mut io_status,
            FSCTL_SET_ZERO_DATA,
            &zero_info as *const _ as *mut _,
            core::mem::size_of::<FILE_ZERO_DATA_INFORMATION>() as u32,
            null_mut(),
            0,
        );
        
        status == STATUS_SUCCESS
    }
}

pub fn create_sparse_file_ntapi(handle: HANDLE, size: u64) -> bool {
    unsafe {
        if !nt_mark_sparse(handle) {
            println!("Failed to mark as sparse");
            return false;
        }
        
        let mut io_status: IO_STATUS_BLOCK = core::mem::zeroed();
        let mut eof_info: FILE_END_OF_FILE_INFORMATION = core::mem::zeroed();
        *eof_info.EndOfFile.QuadPart_mut() = size as _;
        
        let status = NtSetInformationFile(
            handle,
            &mut io_status,
            &eof_info as *const _ as *mut _,
            core::mem::size_of::<FILE_END_OF_FILE_INFORMATION>() as u32,
            FileEndOfFileInformation,
        );
        
        if status != STATUS_SUCCESS {
            println!("Failed to set file size");
            return false;
        }
        
        let mut zero_info: FILE_ZERO_DATA_INFORMATION = core::mem::zeroed();
        *zero_info.FileOffset.QuadPart_mut() = 0;
        *zero_info.BeyondFinalZero.QuadPart_mut() = size as i64;
        
        let status = NtFsControlFile(
            handle,
            null_mut(),
            None,
            null_mut(),
            &mut io_status,
            FSCTL_SET_ZERO_DATA,
            &zero_info as *const _ as *mut _,
            core::mem::size_of::<FILE_ZERO_DATA_INFORMATION>() as u32,
            null_mut(),
            0,
        );
        
        if status != STATUS_SUCCESS {
            println!("Failed to zero file range");
            return false;
        }
        
        true
    }
}

pub fn nt_set_file_pointer(
    handle: HANDLE,
    distance: i64,
    new_position: Option<&mut i64>,
    move_method: u32,
) -> bool {
    unsafe {
        let mut io_status: IO_STATUS_BLOCK = core::mem::zeroed();
        
        if let Some(pos) = new_position {
            let mut pos_info: FILE_POSITION_INFORMATION = core::mem::zeroed();
            let status = NtQueryInformationFile(
                handle,
                &mut io_status,
                &mut pos_info as *mut _ as *mut _,
                core::mem::size_of::<FILE_POSITION_INFORMATION>() as u32,
                FilePositionInformation,
            );
            
            if status != STATUS_SUCCESS {
                return false;
            }
            
            let current = pos_info.CurrentByteOffset.QuadPart();
            let new_pos = match move_method {
                0 => distance,          // FILE_BEGIN
                1 => current + distance, // FILE_CURRENT
                2 => current + distance, // FILE_END (you'd need file size first)
                _ => return false,
            };
            
            *pos = new_pos;
        }
        
        let mut new_pos_info: FILE_POSITION_INFORMATION = core::mem::zeroed();
        *new_pos_info.CurrentByteOffset.QuadPart_mut() = distance;
        
        let status = NtSetInformationFile(
            handle,
            &mut io_status,
            &mut new_pos_info as *mut _ as *mut _,
            core::mem::size_of::<FILE_POSITION_INFORMATION>() as u32,
            FilePositionInformation,
        );
        
        status == STATUS_SUCCESS
    }
}

pub fn nt_set_end_of_file(handle: HANDLE) -> bool {
    unsafe {
        let mut io_status: IO_STATUS_BLOCK = core::mem::zeroed();
        
        let mut std_info: FILE_STANDARD_INFORMATION = core::mem::zeroed();
        let status = NtQueryInformationFile(
            handle,
            &mut io_status,
            &mut std_info as *mut _ as *mut _,
            core::mem::size_of::<FILE_STANDARD_INFORMATION>() as u32,
            FileStandardInformation,
        );
        
        if status != STATUS_SUCCESS {
            return false;
        }
        
        let mut eof_info = FILE_END_OF_FILE_INFORMATION {
            EndOfFile: std_info.EndOfFile,
        };
        
        let status = NtSetInformationFile(
            handle,
            &mut io_status,
            &mut eof_info as *mut _ as *mut _,
            core::mem::size_of::<FILE_END_OF_FILE_INFORMATION>() as u32,
            FileEndOfFileInformation,
        );
        
        if status != STATUS_SUCCESS {
            return false;
        }
        
        let mut pos_info = FILE_POSITION_INFORMATION {
            CurrentByteOffset: std_info.EndOfFile,
        };
        
        let status = NtSetInformationFile(
            handle,
            &mut io_status,
            &mut pos_info as *mut _ as *mut _,
            core::mem::size_of::<FILE_POSITION_INFORMATION>() as u32,
            FilePositionInformation,
        );
        
        status == STATUS_SUCCESS
    }
}

pub fn nt_set_valid_data(handle: HANDLE, size: u64) -> bool {
    unsafe {
        let mut io_status: IO_STATUS_BLOCK = core::mem::zeroed();
        let mut info: FILE_VALID_DATA_LENGTH_INFORMATION = core::mem::zeroed();
        *info.ValidDataLength.QuadPart_mut() = size as _;
        
        let status = NtSetInformationFile(
            handle,
            &mut io_status,
            &info as *const _ as *mut _,
            core::mem::size_of::<FILE_VALID_DATA_LENGTH_INFORMATION>() as u32,
            FileValidDataLengthInformation, // 
        );
        
        status == STATUS_SUCCESS
    }
}

pub fn extend_file_via_ntapi(handle: HANDLE, size: u64) -> bool {
    unsafe {
        let mut io_status: IO_STATUS_BLOCK = core::mem::zeroed();
        let mut new_pos_info: FILE_POSITION_INFORMATION = core::mem::zeroed();
        *new_pos_info.CurrentByteOffset.QuadPart_mut() = size as _;
        
        let status = NtSetInformationFile(
            handle,
            &mut io_status,
            &mut new_pos_info as *mut _ as *mut _,
            core::mem::size_of::<FILE_POSITION_INFORMATION>() as u32,
            FilePositionInformation,
        );

        if status != STATUS_SUCCESS {
            println!("NtSetInformationFile (EndOfFile) failed: {:X}", status);
            return false;
        }

        let mut eof_info = FILE_END_OF_FILE_INFORMATION {
            EndOfFile: new_pos_info.CurrentByteOffset,
        };
        
        let status = NtSetInformationFile(
            handle,
            &mut io_status,
            &mut eof_info as *mut _ as *mut _,
            core::mem::size_of::<FILE_END_OF_FILE_INFORMATION>() as u32,
            FileEndOfFileInformation,
        );

        if status != STATUS_SUCCESS {
            println!("NtSetInformationFile (Position) failed: {:X}", status);
            return false;
        }

        let mut io_status: IO_STATUS_BLOCK = core::mem::zeroed();
        let mut info: FILE_VALID_DATA_LENGTH_INFORMATION = core::mem::zeroed();
        *info.ValidDataLength.QuadPart_mut() = size as _;
        
        let status = NtSetInformationFile(
            handle,
            &mut io_status,
            &info as *const _ as *mut _,
            core::mem::size_of::<FILE_VALID_DATA_LENGTH_INFORMATION>() as u32,
            FileValidDataLengthInformation, // 
        );

        if status != STATUS_SUCCESS {
            println!("NtSetInformationFile (ValidData) failed: {:X}", status);
            return false;
        }
    }
    true
}

pub fn extend_file(handle: HANDLE, size: u64) -> bool {
    unsafe {
        let mut distance: LARGE_INTEGER = core::mem::zeroed();
        *distance.QuadPart_mut() = size as _;
        let mut new_pointer: LARGE_INTEGER = core::mem::zeroed();
        
        if SetFilePointerEx(handle, distance, &mut new_pointer, 0) == 0 {
            let error = GetLastError();
            println!("SetFilePointerEx failed with error: {}", error);
            return false;
        }
        
        if SetEndOfFile(handle) == 0 {
            let error = GetLastError();
            println!("SetEndOfFile failed with error: {}", error);
            return false;
        }

        if SetFileValidData(handle, size as _)  == 0 {
            let error = GetLastError();
            println!("SetEndOfFile failed with error: {}", error);
            return false;
        }
        
        true
    }
}
