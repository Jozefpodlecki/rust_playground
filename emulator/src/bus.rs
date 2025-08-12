use anyhow::{Result, bail};
use std::{rc::Rc, cell::RefCell};

use crate::memory_region::MemoryRegion;

pub type SharedBus = Rc<RefCell<Bus>>;

#[derive(Debug, Default)]
pub struct Bus {
    regions: Vec<MemoryRegion>,
}

impl Bus {
    pub fn new() -> Self {
        Self { regions: vec![] }
    }

    pub fn add_region(&mut self, region: MemoryRegion) {
        self.regions.push(region);
    }

    fn find_region_mut(&mut self, addr: u64) -> Result<&mut MemoryRegion> {
        self.regions
            .iter_mut()
            .find(|r| addr >= r.start_addr && addr < r.end_addr)
            .ok_or_else(|| anyhow::anyhow!("No memory region for address {:#x}", addr))
    }

    fn find_region(&self, addr: u64) -> Result<&MemoryRegion> {
        self.regions
            .iter()
            .find(|r| addr >= r.start_addr && addr < r.end_addr)
            .ok_or_else(|| anyhow::anyhow!("No memory region for address {:#x}", addr))
    }

    pub fn read_exact(&self, addr: u64, buf: &mut [u8]) -> Result<()> {
        let end_addr = addr.checked_add(buf.len() as u64)
            .ok_or_else(|| anyhow::anyhow!("Address overflow in read_exact"))?;

        let region = self.regions.iter()
            .find(|region| region.start_addr <= addr && region.end_addr >= end_addr)
            .ok_or_else(|| anyhow::anyhow!("Memory read_exact out of bounds: {:#x}..{:#x}", addr, end_addr))?;

        let offset = (addr - region.start_addr) as usize;

        buf.copy_from_slice(&region.data[offset..offset + buf.len()]);

        Ok(())
    }

    pub fn read_u64(&self, addr: u64) -> Result<u64> {
        for region in &self.regions {
            if addr >= region.start_addr && addr + 8 <= region.end_addr {
                let offset = (addr - region.start_addr) as usize;
                let mut val = 0u64;
                for i in 0..8 {
                    val |= (region.data[offset + i] as u64) << (i * 8);
                }
                return Ok(val);
            }
        }
        bail!("read_u64: Address {:#x} out of mapped memory regions", addr)
    }

    pub fn write_u64(&mut self, addr: u64, val: u64) -> Result<()> {
        for region in &mut self.regions {
            if addr >= region.start_addr && addr + 8 <= region.end_addr {
                let offset = (addr - region.start_addr) as usize;
                for i in 0..8 {
                    region.data[offset + i] = ((val >> (i * 8)) & 0xFF) as u8;
                }
                return Ok(());
            }
        }
        bail!("write_u64: Address {:#x} out of mapped memory regions", addr)
    }

    pub fn read_u32(&self, addr: u64) -> Result<u32> {
        for region in &self.regions {
            if addr >= region.start_addr && addr + 4 <= region.end_addr {
                let offset = (addr - region.start_addr) as usize;
                let mut val = 0u32;
                for i in 0..4 {
                    val |= (region.data[offset + i] as u32) << (i * 8);
                }
                return Ok(val);
            }
        }
        bail!("read_u32: Address {:#x} out of mapped memory regions", addr)
    }

    pub fn write_u32(&mut self, addr: u64, val: u32) -> Result<()> {
        for region in &mut self.regions {
            if addr >= region.start_addr && addr + 4 <= region.end_addr {
                let offset = (addr - region.start_addr) as usize;
                for i in 0..4 {
                    region.data[offset + i] = ((val >> (i * 8)) & 0xFF) as u8;
                }
                return Ok(());
            }
        }
        bail!("write_u32: Address {:#x} out of mapped memory regions", addr)
    }

    pub fn read_u16(&self, addr: u64) -> Result<u16> {
        for region in &self.regions {
            if addr >= region.start_addr && addr + 2 <= region.end_addr {
                let offset = (addr - region.start_addr) as usize;
                let mut val = 0u16;
                for i in 0..2 {
                    val |= (region.data[offset + i] as u16) << (i * 8);
                }
                return Ok(val);
            }
        }
        bail!("read_u16: Address {:#x} out of mapped memory regions", addr)
    }

    pub fn write_u16(&mut self, addr: u64, val: u16) -> Result<()> {
        for region in &mut self.regions {
            if addr >= region.start_addr && addr + 2 <= region.end_addr {
                let offset = (addr - region.start_addr) as usize;
                for i in 0..2 {
                    region.data[offset + i] = ((val >> (i * 8)) & 0xFF) as u8;
                }
                return Ok(());
            }
        }
        bail!("write_u16: Address {:#x} out of mapped memory regions", addr)
    }

    pub fn read_u8(&self, addr: u64) -> Result<u8> {
        self.find_region(addr)?.read_u8(addr)
    }

    pub fn write_u8(&mut self, addr: u64, val: u8) -> Result<()> {
        self.find_region_mut(addr)?.write_u8(addr, val)
    }

    pub fn write_bytes(&mut self, addr: u64, bytes: &[u8]) -> Result<()> {
        self.find_region_mut(addr)?.write_bytes(addr, bytes)
    }
}
