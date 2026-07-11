use core::{fmt, ptr};
use crate::io::Write;
use crate::error::FileError;
use crate::println;

pub const DEFAULT_BUF_SIZE: usize = 8192;

pub struct BufWriter<W: Write> {
    inner: W,
    buf: [u8; DEFAULT_BUF_SIZE],
    len: usize,
    panicked: bool,
}

impl<W: Write> BufWriter<W> {
    pub fn new(inner: W) -> Self {
        Self {
            inner,
            buf: [0u8; DEFAULT_BUF_SIZE],
            len: 0,
            panicked: false,
        }
    }

    pub fn flush_buf(&mut self) -> Result<(), FileError> {
        let mut written = 0;
        println!("flush_buf {} {}", written, self.len);
        while written < self.len {
            println!("written < self.len");
            self.panicked = true;
            let result = self.inner.write(&self.buf[written..self.len]);
            self.panicked = false;

            match result {
                Ok(0) => return Err(FileError::BufferOverflow),
                Ok(n) => written += n,
                Err(e) => return Err(e),
            }
        }

        self.len = 0;
        Ok(())
    }

    pub fn get_ref(&self) -> &W {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buf[..self.len]
    }

    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    fn spare_capacity(&self) -> usize {
        self.buf.len() - self.len
    }

    unsafe fn write_to_buffer_unchecked(&mut self, buf: &[u8]) {
        let dst = self.buf.as_mut_ptr().add(self.len);
        ptr::copy_nonoverlapping(buf.as_ptr(), dst, buf.len());
        println!("buf.len() {}", buf.len());
        self.len += buf.len();
        println!("len {}", self.len)
    }

    fn write_cold(&mut self, buf: &[u8]) -> Result<usize, FileError> {
        if buf.len() > self.spare_capacity() {
            println!("write_cold");
            self.flush_buf()?;
        }

        if buf.len() >= self.buf.len() {
            self.panicked = true;
            let r = self.inner.write(buf);
            self.panicked = false;
            r
        } else {
            unsafe {
                self.write_to_buffer_unchecked(buf);
            }
            Ok(buf.len())
        }
    }
}

impl<W: Write> Write for BufWriter<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, FileError> {
        if buf.len() < self.spare_capacity() {
            unsafe {
                self.write_to_buffer_unchecked(buf);
            }
            Ok(buf.len())
        } else {
            self.write_cold(buf)
        }
    }

    fn write_all(&mut self, mut buf: &[u8]) -> Result<(), FileError> {

        if buf.len() < self.spare_capacity() {
            unsafe {
                self.write_to_buffer_unchecked(buf);
            }
            Ok(())
        } else {
            while !buf.is_empty() {
                if buf.len() < self.spare_capacity() {
                    unsafe {
                        self.write_to_buffer_unchecked(buf);
                    }
                    break;
                }
                
                println!("!buf.is_empty() ");
                self.flush_buf()?;
                
                if buf.len() >= self.buf.len() {
                    self.panicked = true;
                    let n = self.inner.write(buf)?;
                    self.panicked = false;
                    if n == 0 {
                        return Err(FileError::BufferOverflow);
                    }
                    buf = &buf[n..];
                } else {
                    unsafe {
                        self.write_to_buffer_unchecked(buf);
                    }
                    break;
                }
            }
            Ok(())
        }
    }
}

impl<W: Write> Drop for BufWriter<W> {
    fn drop(&mut self) {
        if !self.panicked {
            println!("drop");
            let _ = self.flush_buf();
        }
    }
}

impl<W: Write + fmt::Debug> fmt::Debug for BufWriter<W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BufWriter")
            .field("writer", &self.inner)
            .field("buffer", &format_args!("{}/{}", self.len, self.buf.len()))
            .finish()
    }
}