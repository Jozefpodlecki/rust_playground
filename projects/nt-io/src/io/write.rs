use utils::println;

use crate::error::FileError;

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize, FileError>;

    fn write_all(&mut self, mut buf: &[u8]) -> Result<(), FileError> {
        while !buf.is_empty() {
            let n = self.write(buf)?;
            if n == 0 {
                return Err(FileError::BufferOverflow);
            }
            buf = &buf[n..];
        }
        Ok(())
    }
}