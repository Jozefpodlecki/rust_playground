use winapi::um::winnt::LARGE_INTEGER;

const NS_PER_TICK: u64 = 100;
const TICKS_PER_SEC: u64 = 1_000_000_000 / NS_PER_TICK;

pub trait DurationExtensions {
    fn to_large_integer(&self) -> LARGE_INTEGER;
}

impl DurationExtensions for core::time::Duration {
    
    #[inline]
    fn to_large_integer(&self) -> LARGE_INTEGER {
        let ticks = (self.as_nanos() / NS_PER_TICK as u128).min(i64::MAX as u128) as i64;

        let mut li: LARGE_INTEGER = unsafe { core::mem::zeroed() };
        unsafe { *li.QuadPart_mut() = -ticks };
        li
    }
}