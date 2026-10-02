use crate::KUserSharedData;

pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn from_shared_data() -> Self {
        let s = KUserSharedData::system_time().0;
        let cookie = KUserSharedData::cookie() as u64;
        let build = KUserSharedData::build_number() as u64;
        let tsc = unsafe { core::arch::x86_64::_rdtsc() };
        let sp = &s as *const u64 as u64;
        Self { state: mix(s ^ cookie.rotate_left(17) ^ build.rotate_left(31) ^ tsc ^ sp) }
    }

    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub const fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        mix(self.state)
    }

    pub const fn random_range(&mut self, range: core::ops::Range<u64>) -> u64 {
        let start = range.start;
        let end = range.end;
        assert!(start < end, "empty range");
        let n = end - start;

        let mut x = self.next_u64();
        let mut m = (x as u128).wrapping_mul(n as u128);
        let mut l = m as u64;
        if l < n {
            let t = n.wrapping_neg() % n;
            while l < t {
                x = self.next_u64();
                m = (x as u128).wrapping_mul(n as u128);
                l = m as u64;
            }
        }
        start + ((m >> 64) as u64)
    }
}

const fn mix(mut x: u64) -> u64 {
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58476D1CE4E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D049BB133111EB);
    x ^ (x >> 31)
}