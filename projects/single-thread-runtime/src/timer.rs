use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::time::Duration;

use ntapi::ntexapi::KUSER_SHARED_DATA;

#[repr(transparent)]
pub struct KUserSharedData(*const KUSER_SHARED_DATA);

static KUSER: KUserSharedData = KUserSharedData((0x7FFE0000 as *const KUSER_SHARED_DATA));

unsafe impl Sync for KUserSharedData {}

impl KUserSharedData {
    fn qpc_frequency(&self) -> i64 {
        unsafe { (*self.0).QpcFrequency }
    }

    fn interrupt_time(&self) -> u64 {
        unsafe {
            let time = (*self.0).InterruptTime;
            ((time.High2Time as u64) << 32) | (time.LowPart as u64)
        }
    }

    fn qpc_time(&self) -> u64 {
        unsafe {
            let qpc_bias = (*self.0).QpcBias;
            self.interrupt_time() + qpc_bias
        }
    }
}

pub struct Delay {
    start: u64,
    duration_ticks: u64,
    elapsed: bool,
}

impl Delay {
    pub fn new(duration: Duration) -> Self {
        let freq = KUSER.qpc_frequency() as u64;
        let duration_ticks = (duration.as_secs_f64() * freq as f64) as u64;
        Self {
            start: 0,
            duration_ticks,
            elapsed: false,
        }
    }
}

impl Future for Delay {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.elapsed {
            return Poll::Ready(());
        }

        if self.start == 0 {
            self.start = KUSER.qpc_time();
        }

        let now = KUSER.qpc_time();
        if now - self.start >= self.duration_ticks {
            self.elapsed = true;
            Poll::Ready(())
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}