use heapless::index_map::FnvIndexMap;
use ntapi::ntexapi::NtDelayExecution;
use winapi::shared::ntdef::HANDLE;
use winapi::um::winnt::LARGE_INTEGER;

use crate::executor::Executor;
use crate::task::{JoinHandle, TaskId};
use core::future::Future;
use core::task::Waker;

static mut RUNTIME: *mut Executor = core::ptr::null_mut();

static mut PENDING_IO_MAP: FnvIndexMap<usize, Waker, 128> = FnvIndexMap::new();

pub struct Runtime {
    executor: Executor,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            executor: Executor::new(),
        }
    }

    pub fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.executor.spawn(future);
    }

    pub fn block_on<F>(&mut self, future: F) -> F::Output
    where
        F: Future + Send + 'static,
        F::Output: Send,
    {
        unsafe {
            RUNTIME = &mut self.executor as *mut Executor;
        }

        let handle = self.executor.spawn_handle(future);

        loop {
            if let Some(result) = handle.try_take() {
                return result;
            }
            self.executor.tick();
            
            switch_to_thread()
        }
    }

    pub fn run(&mut self) {
        self.executor.run();
    }
}

pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    unsafe {
        let executor = &mut *RUNTIME;
        executor.spawn_handle(future)
    }
}

pub fn switch_to_thread() {
    unsafe {
        let mut delay: LARGE_INTEGER = core::mem::zeroed();
        *delay.QuadPart_mut() = 0;
        NtDelayExecution(0, &mut delay);
    };
}

pub fn register_pending_io(handle: HANDLE, waker: Waker) {
    unsafe { PENDING_IO_MAP[&(handle as usize)] = waker; }
}

pub fn complete_io(handle: HANDLE) -> Option<Waker> {
    unsafe {
        PENDING_IO_MAP.remove(&(handle as usize))
    }
}