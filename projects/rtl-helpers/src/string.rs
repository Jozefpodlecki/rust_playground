use core::{fmt, mem::zeroed};

use ntapi::ntrtl::{RtlCreateUnicodeString, RtlCreateUnicodeStringFromAsciiz, RtlFreeUnicodeString, RtlInitUnicodeString, RtlInitUnicodeStringEx};
use winapi::shared::{ntdef::{NTSTATUS, UNICODE_STRING}, ntstatus::STATUS_SUCCESS};

pub struct UnicodeString(UNICODE_STRING);

impl fmt::Debug for UnicodeString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slice = unsafe {
            let len = self.0.Length as usize / 2;
            core::slice::from_raw_parts(self.0.Buffer, len)
        };
    
        f.debug_struct("UnicodeString")
            .field("Length", &self.0.Length)
            .field("MaximumLength", &self.0.MaximumLength)
            .field("Buffer", &format_args!("{:p}", self.0.Buffer))
            .finish()
    }
}

impl UnicodeString {
    pub fn from_str(str: &str) -> Option<Self> {
        unsafe {
            let mut uc_str: UNICODE_STRING = zeroed();
            let success = RtlCreateUnicodeStringFromAsciiz(&mut uc_str, str.as_ptr() as *mut i8);

            if success == 0 {
                return None;
            }

            Some(Self(uc_str))
        }
    }

    pub fn new(str: &[u16]) -> Self {
        unsafe {
            let mut uc_str: UNICODE_STRING = zeroed();
            RtlInitUnicodeString(&mut uc_str, str.as_ptr());

            Self(uc_str)
        }
    }

    pub fn create(str: &[u16]) -> Option<Self> {
        unsafe {
            let mut uc_str: UNICODE_STRING = zeroed();
            let success = RtlCreateUnicodeString(&mut uc_str, str.as_ptr());

            if success == 0 {
                return None;
            }

            Some(Self(uc_str))
        }
    }

    pub fn new_ex(str: &[u16]) -> Result<Self, NTSTATUS> {
        unsafe {
            let mut uc_str: UNICODE_STRING = zeroed();
            let status = RtlInitUnicodeStringEx(&mut uc_str, str.as_ptr());

            if status != STATUS_SUCCESS {
                return Err(status);
            }

            Ok(Self(uc_str))
        }
    }
}

impl Drop for UnicodeString {
    fn drop(&mut self) {
        unsafe { RtlFreeUnicodeString(&mut self.0); }
    }
}

//    fn RtlGetExePath() -> PWSTR;
//     fn RtlGetNtSystemRoot() -> PWSTR;

// unsafe extern "system" {
//     #[link_name = "RtlUTF8StringToUnicodeString"]
//     pub unsafe fn RtlUTF8StringToUnicodeString(
//         DestinationString: *mut UNICODE_STRING,
//         SourceString: *mut CSTRING,
//         AllocateDestinationString: u8, // BOOLEAN is a u8 in Windows
//     ) -> NTSTATUS;
// }

// pub fn utf8_to_unicode_string(input: &str) -> Result<UNICODE_STRING, NTSTATUS> {
//     unsafe {
//         let mut utf8_string = CSTRING {
//             Length: input.len() as u16,
//             MaximumLength: input.len() as u16,
//             Buffer: input.as_ptr() as *mut i8,
//         };
        
//         let mut unicode_string: UNICODE_STRING = core::mem::zeroed();
        
//         let status = RtlUTF8StringToUnicodeString(
//             &mut unicode_string,
//             &mut utf8_string,
//             1, // AllocateDestinationString = TRUE
//         );
//         println!("RtlUTF8StringToUnicodeString {status}");
//         if status == STATUS_SUCCESS {
//             Ok(unicode_string)
//         } else {
//             Err(status)
//         }
//     }
// }

// pub fn utf8_to_utf16<const N: usize>(input: &str) -> Result<[u16; N], NTSTATUS> {
//     let input_bytes = input.as_bytes();
//     let max_needed = (input_bytes.len() + 1) * 2;
    
//     if N * 2 < max_needed {
//         return Err(STATUS_BUFFER_TOO_SMALL);
//     }
    
//     let mut output = [0u16; N];
//     let mut actual_bytes: u32 = 0;
    
//     let status = unsafe {
//         RtlUTF8ToUnicodeN(
//             output.as_mut_ptr(),
//             (N * 2) as u32, // Max bytes to write
//             &mut actual_bytes,
//             input_bytes.as_ptr() as *const i8,
//             input_bytes.len() as u32,
//         )
//     };
    
//     if status == STATUS_SUCCESS {
//         let actual_len = (actual_bytes / 2) as usize;
//         Ok(output)
//     } else {
//         Err(status)
//     }
// }

// pub fn delete_file_nt(path: &str) -> Result<(), NTSTATUS> {
//     unsafe {
//         let mut unicode_string = utf8_to_unicode_string(path)?;
//         println!("unicode_string");
//         let mut attributes = OBJECT_ATTRIBUTES {
//             Length: core::mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
//             RootDirectory: core::ptr::null_mut(),
//             ObjectName: &mut unicode_string,
//             Attributes: OBJ_CASE_INSENSITIVE,
//             SecurityDescriptor: core::ptr::null_mut(),
//             SecurityQualityOfService: core::ptr::null_mut(),
//         };

//         let status = NtDeleteFile(&mut attributes);

//         if status == STATUS_SUCCESS {
//             Ok(())
//         } else {
//             Err(status)
//         }
//     }
// }