mod types;
mod options;
mod file;

use core::ptr::null_mut;

use ntapi::ntioapi::NtDeleteFile;
pub use types::*;
pub use file::*;
use toolkit::{U16CStackString, println};
use winapi::shared::ntdef::{OBJ_CASE_INSENSITIVE, OBJECT_ATTRIBUTES};

use crate::{error::FileError, io::{Read, Seek, Write}};

pub fn remove_file(path: &str) -> Result<(), FileError> {
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
    
    let status = unsafe { toolkit::syscalls::NtDeleteFile(&mut object_attributes) };
    // let status = unsafe { NtDeleteFile(&mut object_attributes) };

     if status >= 0 {
        Ok(())
    } else {
        Err(FileError::from(status))
    }
}

pub fn manual_copy(src: &str, dest: &str) -> Result<u64, FileError> {
    let mut src = File::open(src)?;
    let mut dest = File::create(dest)?;
    let mut buf = [0u8; 65536];
    let mut total = 0;

    loop {
        let n = match src.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(FileError::EndOfFile) => break,
            Err(err) => return Err(err),
        };

        dest.write_all(&buf[..n])?;

        total += n as u64;
    }

    Ok(total)
}