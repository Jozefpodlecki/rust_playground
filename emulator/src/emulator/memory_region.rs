use anyhow::{Result, bail};
use bincode::{Decode, Encode};

#[derive(Debug, Encode, Decode, Clone)]
pub struct MemoryRegion {
    pub start_addr: u64,
    pub end_addr: u64,
    pub data: Vec<u8>,
}

impl MemoryRegion {
    pub fn new(start_addr: u64, size: usize) -> Self {
        Self {
            start_addr,
            end_addr: start_addr + size as u64,
            data: vec![0; size],
        }
    }

    fn to_index(&self, addr: u64) -> Result<usize> {
        if addr < self.start_addr || addr >= self.end_addr {
            bail!("Address {:#x} out of range [{:#x}..{:#x})",
                addr, self.start_addr, self.end_addr );
        }
        Ok((addr - self.start_addr) as usize)
    }

    pub fn read_u8(&self, addr: u64) -> Result<u8> {
        Ok(self.data[self.to_index(addr)?])
    }

    pub fn write_u8(&mut self, addr: u64, val: u8) -> Result<()> {
        let idx = self.to_index(addr)?;
        self.data[idx] = val;
        Ok(())
    }

    pub fn write_bytes(&mut self, addr: u64, bytes: &[u8]) -> Result<()> {
        let idx = self.to_index(addr)?;
        if idx + bytes.len() > self.data.len() {
            bail!("Write OOB at {:#x}", addr);
        }
        self.data[idx..idx + bytes.len()].copy_from_slice(bytes);
        Ok(())
    }
}
