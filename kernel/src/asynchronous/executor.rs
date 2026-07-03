use alloc::{collections::btree_map::BTreeMap, sync::Arc};
use core::task::{Context, Waker};
use crossbeam_queue::ArrayQueue;

use super::{
    task::{Task, TaskId},
    waker::TaskWaker,
};

const MAX_CONCURRENT_TASK_COUNT: usize = 100;

pub struct AsyncExecutor {
    tasks: BTreeMap<TaskId, Task>,
    task_queue: Arc<ArrayQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl AsyncExecutor {
    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        log::debug!("Spawning new task {:?}", task_id);

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

    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(MAX_CONCURRENT_TASK_COUNT)),
            waker_cache: BTreeMap::new(),
        }
    }

    pub fn run_ready_tasks(&mut self) {
        let Self {
            waker_cache,
            task_queue,
            tasks,
        } = self;

        while let Some(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                // Skip, as task no longer exists
                None => continue,
            };

            let waker = waker_cache
                .entry(task_id)
                .or_insert(TaskWaker::waker(task_id, task_queue.clone()));
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                core::task::Poll::Ready(_) => {
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                core::task::Poll::Pending => {}
            }
        }
    }

    fn sleep_if_idle(&self) {
        use x86_64::instructions::interrupts;

        interrupts::disable();

        if self.task_queue.is_empty() {
            interrupts::enable();
            x86_64::instructions::hlt();
        } else {
            interrupts::enable();
        }
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }
}
