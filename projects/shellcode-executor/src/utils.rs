use core::fmt;

pub struct OctaDisplay<'a>(&'a [u16]);

impl<'a> OctaDisplay<'a> {
    pub fn new(slice: &'a [u16]) -> impl Iterator<Item = Self> {
        slice.chunks(8).map(OctaDisplay)
    }
}

impl<'a> fmt::Display for OctaDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\".octa 0x")?;

        let n = self.0.len();
        let total_bytes = n * 2;
        let pad = 16 - total_bytes;

        for _ in 0..pad {
            write!(f, "00")?;
        }

        for &c in self.0.iter().rev() {
            write!(f, "{:02X}{:02X}", (c >> 8) as u8, (c & 0xFF) as u8)?;
        }

        write!(f, "\"")?;

        Ok(())
    }
}