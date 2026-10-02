use core::io::{Write, Error as IoError};

pub struct BufWriter<W: Write, const N: usize> {
    inner: W,
    buf: [u8; N],
    len: usize,
}

impl<W: Write, const N: usize> BufWriter<W, N> {
    pub fn new(inner: W) -> Self {
        Self {
            inner,
            buf: [0u8; N],
            len: 0,
        }
    }

    pub fn get_ref(&self) -> &W {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }

    fn flush_buf(&mut self) -> Result<(), IoError> {
        if self.len == 0 {
            return Ok(());
        }
        self.inner.write_all(&self.buf[..self.len])?;
        self.len = 0;
        Ok(())
    }
}

impl<W: Write, const N: usize> Write for BufWriter<W, N> {
    fn write(&mut self, data: &[u8]) -> Result<usize, IoError> {
        if data.len() >= N {
            self.flush_buf()?;
            return self.inner.write(data);
        }

        if self.len + data.len() > N {
            self.flush_buf()?;
        }

        self.buf[self.len..self.len + data.len()].copy_from_slice(data);
        self.len += data.len();
        Ok(data.len())
    }

    fn write_all(&mut self, data: &[u8]) -> Result<(), IoError> {
        if data.len() >= N {
            self.flush_buf()?;
            return self.inner.write_all(data);
        }

        if self.len + data.len() > N {
            self.flush_buf()?;
        }

        self.buf[self.len..self.len + data.len()].copy_from_slice(data);
        self.len += data.len();
        Ok(())
    }

    fn flush(&mut self) -> Result<(), IoError> {
        self.flush_buf()?;
        self.inner.flush()
    }
}

impl<W: Write, const N: usize> Drop for BufWriter<W, N> {
    fn drop(&mut self) {
        let _ = self.flush_buf();
    }
}