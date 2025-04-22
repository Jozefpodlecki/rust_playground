use std::time::{Duration, Instant};

pub struct IntervalTimer {
    interval: Duration,
    last_tick: Instant,
}

impl IntervalTimer {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            last_tick: Instant::now(),
        }
    }

    pub fn set_interval(&mut self, interval: Duration) {
        self.interval = interval;
    }

    pub fn tick_if_elapsed(&mut self) -> bool {
        if self.last_tick.elapsed() >= self.interval {
            self.last_tick = Instant::now();
            true
        } else {
            false
        }
    }
}