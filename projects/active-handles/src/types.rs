use core::{mem, panic::PanicInfo, ptr, slice};

use hashbrown::HashSet;
use ntapi::{ntobapi::NtClose, ntpsapi::{NtQueryInformationProcess, ProcessImageFileNameWin32}};
use toolkit::{ProcessMemoryReader, ProcessQuerier, U8CStackString, println};
use winapi::{shared::{ntdef::{HANDLE, NTSTATUS, UNICODE_STRING}, ntstatus::{STATUS_ACCESS_VIOLATION, STATUS_SUCCESS}}, um::{handleapi::CloseHandle, winnt::{PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION}}} ;


#[derive(Hash, PartialEq, Eq)]
pub struct FileName {
    buffer: [u8; 512],
    len: usize,
}

impl FileName {
    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.buffer[..self.len]) }
    }
}

pub fn file_name(handle: HANDLE) -> Result<FileName, NTSTATUS> {
    unsafe {
        let mut return_length = 0;
        let mut buffer: [u8; 512] = [0u8; 512];
        
        let status = NtQueryInformationProcess(
            handle,
            ProcessImageFileNameWin32,
            buffer.as_mut_ptr() as *mut _,
            buffer.len() as u32,
            &mut return_length,
        );

        if status != STATUS_SUCCESS {
            return Err(status);
        }

        let unicode_string = &*(buffer.as_ptr() as *const UNICODE_STRING);
        let u16_count = unicode_string.Length as usize / 2;
        let start = mem::size_of::<UNICODE_STRING>();
        
        let u16_slice = core::slice::from_raw_parts(
            buffer.as_ptr().add(start) as *const u16,
            u16_count,
        );
        
        let utf16_chars = char::decode_utf16(u16_slice.iter().copied());
        let mut utf8_buffer: [u8; 512] = [0u8; 512];
        let mut pos = 0;
        
        for c in utf16_chars {
            if let Ok(c) = c {
                let mut temp = [0u8; 4];
                let encoded = c.encode_utf8(&mut temp);
                let bytes = encoded.as_bytes();
                if pos + bytes.len() <= 512 {
                    utf8_buffer[pos..pos + bytes.len()].copy_from_slice(bytes);
                    pos += bytes.len();
                } else {
                    break;
                }
            }
        }

        Ok(FileName {
            buffer: utf8_buffer,
            len: pos,
        })
    }
}