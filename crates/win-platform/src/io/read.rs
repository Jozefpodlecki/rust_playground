use crate::io::FileError;

pub trait Sink {
    fn write_chunk(&mut self, data: &[u8]) -> Result<(), FileError>;
    fn finish(&mut self, total: usize) -> Result<usize, FileError> {
        Ok(total)
    }
}

#[cfg(feature = "alloc")]
impl Sink for alloc::vec::Vec<u8> {
    fn write_chunk(&mut self, data: &[u8]) -> Result<(), FileError> {
        self.extend_from_slice(data);
        Ok(())
    }
}

impl Sink for &mut [u8] {
    fn write_chunk(&mut self, data: &[u8]) -> Result<(), FileError> {
        if data.len() > self.len() {
            return Err(FileError::BufferTooSmall);
        }
        let (head, tail) = core::mem::take(self).split_at_mut(data.len());
        head.copy_from_slice(data);
        *self = tail;
        Ok(())
    }
}

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, FileError>;

    fn read_to_end(&mut self, buf: impl Sink) -> Result<usize, FileError> {
        let mut sink = buf;
        let mut total = 0usize;
        let mut temp = [0u8; 4096];
        loop {
            match self.read(&mut temp) {
                Ok(0) => break,
                Ok(n) => {
                    sink.write_chunk(&temp[..n])?;
                    total += n;
                }
                Err(FileError::EndOfFile) => break,
                Err(e) => return Err(e),
            }
        }
        sink.finish(total)
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