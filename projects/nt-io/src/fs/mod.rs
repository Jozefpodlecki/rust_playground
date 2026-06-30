mod types;
mod options;
mod file;

pub use types::*;
pub use file::*;
use utils::println;

use crate::{error::FileError, io::{Read, Seek, Write}};

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

    src.seek(crate::io::SeekFrom::Start(0))?;
    dest.seek(crate::io::SeekFrom::Start(0))?;

    let mut buf = [0u8; 65536];
    let mut total = 0;

    loop {

        let n = match src.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(FileError::EndOfFile) => break,
            Err(e) => return Err(e),
        };

        dest.write_all(&buf[..n])?;

        total += n as u64;
    }

    Ok(total)
}