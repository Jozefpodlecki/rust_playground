use core::sync::atomic::{AtomicU64, Ordering};

use crate::arena::Pos;

#[repr(transparent)]
pub struct Rng(AtomicU64);

impl Rng {
    pub const fn new(seed: u64) -> Self {
        Self(AtomicU64::new(seed))
    }

    pub fn next(&self) -> u64 {
        let mut x = self.0.load(Ordering::Relaxed);
        if x == 0 {
            x = 0x9E3779B97F4A7C15;
        }
        x ^= x << 7;
        x ^= x >> 9;
        self.0.store(x, Ordering::Relaxed);
        x
    }

    pub fn range(&self, min: usize, max: usize) -> usize {
        let range = max - min;
        if range == 0 {
            return min;
        }
        min + (self.next() as usize % range)
    }

    pub fn pos(&self) -> Pos {
        Pos(
            self.range(2, crate::arena::SIZE - 2),
            self.range(2, crate::arena::SIZE - 2),
        )
    }

    pub fn pos_near(&self, center: Pos, min_dist: usize, max_dist: usize) -> Pos {
        loop {
            let pos = self.pos();
            let dist = pos.distance_to(center);
            if dist >= min_dist && dist <= max_dist {
                return pos;
            }
        }
    }
}