mod types;
mod options;
mod buf_writer;
mod file;

use core::ptr::null_mut;

use heapless::Vec;
use ntapi::ntrtl::{RtlDosPathNameToNtPathName_U, RtlFreeUnicodeString};
pub use types::*;
pub use buf_writer::*;
pub use file::*;
use winapi::shared::ntdef::UNICODE_STRING;

use crate::{U16CStackString, error::FileError, io::{Read, Seek, Write}};

pub fn read<const N: usize, const M: usize>(path: U16CStackString<N>) -> Result<Vec<u8, M>, FileError> {
    let mut file = File::open(path).unwrap();
    let mut buffer: Vec<u8, M> = Vec::new();
    file.read_to_end_fixed(&mut buffer)?;
    Ok(buffer)
}

pub fn manual_copy(src: &str, dest: &str) -> Result<u64, FileError> {
    let src = U16CStackString::<260>::from_str(src).unwrap();
    let mut src = File::open(src)?;
    let dest = U16CStackString::<260>::from_str(dest).unwrap();
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

pub fn canonicalize<const N: usize, const M: usize>(mut path: U16CStackString<N>) -> Option<U16CStackString<M>> {
    
    let mut nt_path: UNICODE_STRING = UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: null_mut(),
    };
    
    let result = unsafe {
        RtlDosPathNameToNtPathName_U(
            path.as_mut_ptr(),
            &mut nt_path as *mut UNICODE_STRING,
            null_mut(),
            null_mut(),
        )
    };

    let canonicalized = U16CStackString::<M>::from_ptr(nt_path.Buffer);

    if result != 0 {
        unsafe { RtlFreeUnicodeString(&mut nt_path);}
    }

    canonicalized
}