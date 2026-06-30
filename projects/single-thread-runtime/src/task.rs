use core::future::Future;
use core::pin::Pin;
use core::task::{RawWaker, RawWakerVTable, Waker};
use alloc::boxed::Box;
use alloc::sync::Arc;
use crate::channel::{Receiver, Sender};
use crate::queue::TaskQueue;

pub type TaskId = u64;

pub struct TaskWaker {
    queue: *mut TaskQueue,
    task_id: TaskId,
}

impl TaskWaker {
    pub fn new(queue: *mut TaskQueue, task_id: TaskId) -> Self {
        Self { queue, task_id }
    }

    fn wake(ptr: *const ()) {
        unsafe {
            let waker = &*(ptr as *const Self);
            if let Some(queue) = waker.queue.as_mut() {
                queue.requeue_by_id(waker.task_id);
            }
        }
    }

    fn wake_by_ref(ptr: *const ()) {
        unsafe {
            let waker = &*(ptr as *const Self);
            if let Some(queue) = waker.queue.as_mut() {
                queue.requeue_by_id(waker.task_id);
            }
        }
    }

    fn clone(ptr: *const ()) -> RawWaker {
        unsafe {
            let waker = &*(ptr as *const Self);
            let cloned = TaskWaker::new(waker.queue, waker.task_id);
            RawWaker::new(
                Arc::into_raw(Arc::new(cloned)) as *const (),
                &Self::VTABLE,
            )
        }
    }

    fn drop(ptr: *const ()) {
        unsafe {
            drop(Arc::from_raw(ptr as *const Self));
        }
    }

    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        Self::clone,
        Self::wake,
        Self::wake_by_ref,
        Self::drop,
    );
}

impl TaskWaker {
    pub fn to_waker(self: Arc<Self>) -> Waker {
        unsafe {
            Waker::from_raw(RawWaker::new(
                Arc::into_raw(self) as *const (),
                &Self::VTABLE,
            ))
        }
    }
}

pub struct Task {
    pub id: TaskId,
    pub future: Pin<Box<dyn Future<Output = ()> + Send>>,
    pub completion: Option<Sender<()>>,
}

impl Task {
    pub fn new<F>(id: TaskId, future: F) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Self {
            id,
            future: Box::pin(future),
            completion: None,
        }
    }

    pub fn with_completion<F, T>(id: TaskId, future: F, tx: Sender<T>) -> Self
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let wrapped = async move {
            let result = future.await;
            tx.send(result);
        };
        
        Self {
            id,
            future: Box::pin(wrapped),
            completion: None,
        }
    }

    pub fn id(&self) -> TaskId {
        self.id
    }

    pub fn poll(&mut self, cx: &mut core::task::Context<'_>) -> core::task::Poll<()> {
        let pinned = unsafe { Pin::new_unchecked(&mut self.future) };
        pinned.poll(cx)
    }
}

pub struct JoinHandle<T> {
    receiver: Receiver<T>,
}

impl<T> JoinHandle<T> {
    pub fn new(receiver: Receiver<T>) -> Self {
        Self { receiver }
    }

    pub fn try_take(&self) -> Option<T> {
        self.receiver.try_recv()
    }
}

impl<T> core::future::Future for JoinHandle<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        if let Some(result) = self.try_take() {
            core::task::Poll::Ready(result)
        } else {
            self.receiver.register_waker(cx.waker());
            core::task::Poll::Pending
        }
    }
}