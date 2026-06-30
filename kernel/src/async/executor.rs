use alloc::collections::btree_map::BTreeMap;
use crossbeam_queue::ArrayQueue;

use crate::r#async::task::{Task, TaskId};

pub struct AsyncExecutor {
    tasks: BTreeMap<TaskId, Task>,
    task_queue: ArrayQueue<TaskId>,
}

impl AsyncExecutor {
    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;

        if self.tasks.insert(task_id, task).is_some() {
            // Panicking here is ok, as we ne know this should never happen
            // as long as there is not a bug with our code
            panic!(
                "Tried to add a task with ID ({:?}) which already exists!",
                task_id
            );
        }

        self.task_queue.push(task_id).expect("Task queue full!");
    }
}
