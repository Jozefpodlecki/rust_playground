use anyhow::{bail, Result};

#[derive(Debug, Default, Clone)]
pub struct Memory {
    data: Vec<u8>,
}

impl Memory {
    pub fn new(size: usize) -> Self {
        Memory {
            data: vec![0; size],
        }
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn read_u8(&self, addr: u64) -> Result<u8> {
        let idx = addr as usize;
        self.data.get(idx).copied().ok_or_else(|| anyhow::anyhow!("Memory read_u8 OOB {:x}", addr))
    }

    pub fn read_u64(&self, addr: u64) -> Result<u64> {
        let idx = addr as usize;
        if idx + 8 > self.data.len() {
            bail!("Memory read_u64 OOB {:x}", addr);
        }
        let mut val = 0u64;
        for i in 0..8 {
            val |= (self.data[idx + i] as u64) << (i * 8);
        }
        Ok(val)
    }

    pub fn write_u8(&mut self, addr: u64, val: u8) -> Result<()> {
        let idx = addr as usize;
        if idx >= self.data.len() {
            bail!("Memory write_u8 OOB {:x}", addr);
        }
        self.data[idx] = val;
        Ok(())
    }

    pub fn write_u64(&mut self, addr: u64, val: u64) -> Result<()> {
        let idx = addr as usize;
        if idx + 8 > self.data.len() {
            bail!("Memory write_u64 OOB {:x}", addr);
        }
        for i in 0..8 {
            self.data[idx + i] = ((val >> (i * 8)) & 0xff) as u8;
        }
        Ok(())
    }

    pub fn write_bytes(&mut self, addr: u64, bytes: &[u8]) -> Result<()> {
        let idx = addr as usize;
        if idx + bytes.len() > self.data.len() {
            bail!("Memory write_bytes OOB {:x} len {}", addr, bytes.len());
        }
        self.data[idx..idx + bytes.len()].copy_from_slice(bytes);
        Ok(())
    }
}
