mod types;
mod options;
mod file;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

pub use types::*;
pub use options::*;
pub use file::*;

use crate::{io::{FileError, Write}, types::UnicodeString};

#[cfg(feature = "alloc")]

#[macro_export]
macro_rules! writeln {
    ($dst:expr $(,)?) => {
        $crate::write!($dst, "\n")
    };
    ($dst:expr, $($arg:tt)*) => {
        $dst.write_fmt($crate::format_args_nl!($($arg)*))
    };
    ($($arg:tt)*) => {
        compile_error!("requires a destination and format arguments, like `writeln!(dest, \"format string\", args...)`")
    };
}

#[cfg(feature = "alloc")]
pub fn read(mut path_uc: crate::types::UnicodeString) -> Result<Vec<u8>, FileError> {
    use crate::io::Read;
    
    let mut file = File::open(path_uc)?;
    let metadata = file.metadata()?;
    // let mut bytes = Vec::with_capacity(metadata.size.0 as _);
    let mut bytes = alloc::vec![0u8; metadata.size.0 as usize];
    file.read_exact(&mut bytes)?;

    Ok(bytes)
}

pub fn write<C: AsRef<[u8]>>(mut path_uc: UnicodeString, contents: C) -> Result<(), FileError> {
    let mut file = File::create(path_uc)?;
    file.write_all(contents.as_ref());

    Ok(())
}
