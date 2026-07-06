use core::{sync::atomic::{AtomicI64, Ordering}, time::Duration};
use core::ops::{Add, Sub};

use winapi::um::{profileapi::{QueryPerformanceCounter, QueryPerformanceFrequency}, winnt::LARGE_INTEGER};

static FREQUENCY: AtomicI64 = AtomicI64::new(0);

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub struct Instant(Duration);

impl Instant {
    pub fn now() -> Instant {

        let freq = frequency() as u64;
        let now = now();
        let instant_nsec = mul_div_u64(now as u64, NANOS_PER_SEC, freq);
        let instant_nsec = instant_nsec + (u64::MAX / 4);

        Self(Duration::from_nanos(instant_nsec))
    }
}

impl Sub<Instant> for Instant {
    type Output = Duration;
    
    fn sub(self, other: Instant) -> Duration {
        if self < other {
            Duration::ZERO
        } else {
            Duration::from_nanos_u128(self.0.as_nanos() - other.0.as_nanos())
        }
    }
}

impl Sub<Duration> for Instant {
    type Output = Instant;
    
    fn sub(self, other: Duration) -> Instant {
        let nanos = self.0.as_nanos().saturating_sub(other.as_nanos());
        Instant(Duration::from_nanos_u128(nanos))
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;
    
    fn add(self, other: Duration) -> Instant {
        Instant(self.0 + other)
    }
}

const NANOS_PER_SEC: u64 = 1_000_000_000;

pub fn mul_div_u64(value: u64, numerator: u64, denom: u64) -> u64 {
    // let q = value / denom;
    // let r = value % denom;
    // q * numerator + r * numerator / denom
    ((value as u128 * numerator as u128) / denom as u128) as u64
}

pub fn now() -> i64 {
    unsafe {
        let mut qpc_value: LARGE_INTEGER = core::mem::zeroed();
        QueryPerformanceCounter(&mut qpc_value);
        *qpc_value.QuadPart()
    }
}

pub fn frequency() -> i64 {

    let cached = FREQUENCY.load(Ordering::Relaxed);
    
    if cached != 0 {
        return cached;
    }
    
    frequency_init(&FREQUENCY)
}

 #[cold]
fn frequency_init(cache: &AtomicI64) -> i64 {
    unsafe {
        let mut frequency: LARGE_INTEGER = core::mem::zeroed();
        QueryPerformanceFrequency(&mut frequency);
        core::hint::assert_unchecked(*frequency.QuadPart() != 0);

        let value = *frequency.QuadPart();
        cache.store(value, Ordering::Relaxed);
        
        value
    }
}