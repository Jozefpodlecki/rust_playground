use crate::task::Task;
use crate::task::TaskId;
use alloc::collections::VecDeque;

pub struct TaskQueue {
    queue: VecDeque<Task>,
}

impl TaskQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn push(&mut self, task: Task) {
        self.queue.push_back(task);
    }

    pub fn pop(&mut self) -> Option<Task> {
        self.queue.pop_front()
    }

    pub fn requeue_by_id(&mut self, id: TaskId) {
        for i in 0..self.queue.len() {
            if self.queue[i].id() == id {
                if let Some(task) = self.queue.remove(i) {
                    self.queue.push_back(task);
                }
                break;
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}