use alloc::boxed::Box;
use core::{
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(u64);

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

impl TaskId {
    fn new() -> Self {
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct Task {
    pub id: TaskId,
    pub future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }
}
