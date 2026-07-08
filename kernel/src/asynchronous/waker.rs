use core::task::Waker;

use super::task::TaskId;
use alloc::{sync::Arc, task::Wake};
use crossbeam_queue::ArrayQueue;

pub struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    pub fn waker(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(Self::new(task_id, task_queue)))
    }

    pub fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Self {
        Self {
            task_id,
            task_queue,
        }
    }

    fn wake_task(&self) {
        #[allow(clippy::expect_used)]
        self.task_queue.push(self.task_id).expect("Task_queue full!");
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}
