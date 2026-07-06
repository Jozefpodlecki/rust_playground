use core::cell::UnsafeCell;

const SPIN_LIMIT: u32 = 6;

pub struct Backoff(UnsafeCell<u32>);

impl Backoff {
    pub fn new() -> Self {
        Self(UnsafeCell::new(0))
    }

    pub fn spin_light(&self) {
        unsafe {
            let step = (*self.0.get()).min(SPIN_LIMIT);
            for _ in 0..step.pow(2) {
                core::hint::spin_loop();
            }

            self.0.replace((*self.0.get()) + 1);
        }
    }

    pub fn spin_heavy(&self) {
        unsafe {
            if (*self.0.get()) <= SPIN_LIMIT {
                for _ in 0..(*self.0.get()).pow(2) {
                    core::hint::spin_loop()
                }
            } else {
                // crate::thread::yield_now();
                // c::SwitchToThread();
            }

            self.0.replace((*self.0.get()) + 1);
        }
    }
}
