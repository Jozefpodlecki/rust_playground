use crate::error::FileError;

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, FileError>;

    #[cfg(feature = "alloc")]
    fn read_to_end(&mut self, buf: &mut alloc::vec::Vec<u8>) -> Result<usize, FileError> {
        let mut total = 0;
        let mut temp = [0u8; 4096];
        loop {
            match self.read(&mut temp) {
                Ok(0) => break,
                Ok(n) => {
                    buf.extend_from_slice(&temp[..n]);
                    total += n;
                }
                Err(FileError::EndOfFile) => return Ok(total),
                Err(err) => return Err(err),
            }
        }
        Ok(total)
    }

    fn read_to_end_fixed<const N: usize>(&mut self, buf: &mut heapless::Vec<u8, N>) -> Result<usize, FileError> {
        let mut total = 0;
        let mut temp = [0u8; 4096];
        loop {
            match self.read(&mut temp) {
                Ok(0) => break,
                Ok(n) => {
                    buf.extend_from_slice(&temp[..n]);
                    total += n;
                }
                Err(FileError::EndOfFile) => return Ok(total),
                Err(err) => return Err(err),
            }
        }
        Ok(total)
    }

    fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<(), FileError> {
        let original_len = buf.len();
        let mut total_read = 0;
        while !buf.is_empty() {
            match self.read(buf) {
                Ok(0) => {
                    return Err(FileError::UnexpectedEof);
                }
                Ok(n) => {
                    total_read += n;
                    buf = &mut buf[n..];
                }
                Err(FileError::EndOfFile) => {
                    return Err(FileError::UnexpectedEof);
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}