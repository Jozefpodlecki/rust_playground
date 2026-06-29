use core::ops::Range;

use crate::chacha::ChaChaRng;

#[link(name = "bcryptprimitives", kind = "raw-dylib")]
unsafe extern "system" {
    pub fn ProcessPrng(pbData: *mut u8, cbData: usize) -> i32;
}

fn seed_from_os() -> [u8; 32] {
    let mut seed = [0u8; 32];
    unsafe { ProcessPrng(seed.as_mut_ptr() as *mut _, seed.len() as usize); }
    seed
}

pub struct Rng {
    inner: ChaChaRng,
}

impl Rng {
    pub fn new() -> Self {
        let seed = seed_from_os();
        Rng {
            inner: ChaChaRng::from_seed(&seed),
        }
    }
    
    pub fn next_u32(&mut self) -> u32 {
        self.inner.next_u32()
    }
    
    pub fn next_u64(&mut self) -> u64 {
        self.inner.next_u64()
    }
    
    pub fn next_f32(&mut self) -> f32 {
        let bits = self.next_u32() & 0x007FFFFF;
        let float_bits = bits | 0x3F800000;
        f32::from_bits(float_bits) - 1.0
    }
    
    pub fn next_f64(&mut self) -> f64 {
        let bits = self.next_u64() & 0x000FFFFFFFFFFFFF;
        let float_bits = bits | 0x3FF0000000000000;
        f64::from_bits(float_bits) - 1.0
    }
    
    pub fn next_bool(&mut self) -> bool {
        self.next_u32() & 1 == 1
    }
    
    pub fn range_u32(&mut self, range: Range<u32>) -> u32 {
        let min = range.start;
        let max = range.end;
        if min >= max {
            panic!("Invalid range");
        }
        
        let range_len = max - min + 1;
        let max_valid = u32::MAX - (u32::MAX % range_len);
        
        loop {
            let value = self.next_u32();
            if value <= max_valid {
                return min + (value % range_len);
            }
        }
    }
    
    
    pub fn range_u64(&mut self, min: u64, max: u64) -> u64 {
        if min >= max {
            panic!("Invalid range");
        }
        
        let range = max - min + 1;
        let max_valid = u64::MAX - (u64::MAX % range);
        
        loop {
            let value = self.next_u64();
            if value <= max_valid {
                return min + (value % range);
            }
        }
    }
    
    pub fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        if min >= max {
            panic!("Invalid range");
        }
        min + (max - min) * self.next_f32()
    }
    
    pub fn range_f64(&mut self, min: f64, max: f64) -> f64 {
        if min >= max {
            panic!("Invalid range");
        }
        min + (max - min) * self.next_f64()
    }
    
    pub fn fill_bytes(&mut self, buffer: &mut [u8]) {
        self.inner.fill_bytes(buffer);
    }
}