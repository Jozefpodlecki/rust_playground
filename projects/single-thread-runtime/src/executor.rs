use crate::task::{Task, JoinHandle, TaskWaker};
use crate::queue::TaskQueue;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use alloc::sync::Arc;

pub struct Executor {
    queue: TaskQueue,
    next_id: u64,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            queue: TaskQueue::new(),
            next_id: 1,
        }
    }

    pub fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let id = self.next_id;
        self.next_id += 1;
        let task = Task::new(id, future);
        self.queue.push(task);
    }

    pub fn spawn_handle<F, T>(&mut self, future: F) -> JoinHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = crate::channel::channel();
        let id = self.next_id;
        self.next_id += 1;
        let task = Task::with_completion(id, future, tx);
        self.queue.push(task);
        JoinHandle::new(rx)
    }

    pub fn tick(&mut self) -> bool {
        if let Some(mut task) = self.queue.pop() {
            let task_id = task.id();
            let queue_ptr = &mut self.queue as *mut TaskQueue;
            
            let waker = Arc::new(TaskWaker::new(queue_ptr, task_id)).to_waker();
            let mut cx = Context::from_waker(&waker);
            
            let pinned = unsafe { Pin::new_unchecked(&mut task.future) };
            if let Poll::Pending = pinned.poll(&mut cx) {
                self.queue.push(task);
            }
            true
        } else {
            false
        }
    }

    pub fn run(&mut self) {
        while !self.queue.is_empty() {
            self.tick();
            crate::runtime::switch_to_thread();
        }
    }
}