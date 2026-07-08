use alloc::{collections::btree_map::BTreeMap, vec::Vec};
use core::{sync::atomic::Ordering, task::Waker};
use spin::Mutex;
use x86_64::instructions::interrupts;

pub static SLEEP_QUEUE: Mutex<BTreeMap<u64, Vec<Waker>>> = Mutex::new(BTreeMap::new());

pub struct Sleep {
    wake_tick: u64,
}

const NANOSECONDS_IN_SECOND: u128 = 1_000_000_000;

impl Sleep {
    pub fn new(duration: core::time::Duration) -> Self {
        let freq = crate::time::MONOTONIC_TICK_FREQUENCY.load(Ordering::Relaxed) as u128;
        if freq == 0 {
            panic!("Sleep frequency is 0!")
        }

        let ticks = (duration.as_nanos() * freq) / NANOSECONDS_IN_SECOND;
        Sleep::ticks(ticks as u64)
    }

    pub fn ticks(ticks: u64) -> Self {
        let now = crate::time::MONOTONIC_TICKS.load(Ordering::Acquire);

        Self {
            wake_tick: now + ticks,
        }
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let now = crate::time::MONOTONIC_TICKS.load(Ordering::Acquire);

        if now >= self.wake_tick {
            return core::task::Poll::Ready(());
        }

        interrupts::without_interrupts(|| {
            let mut queue = SLEEP_QUEUE.lock();
            queue
                .entry(self.wake_tick)
                .or_default()
                .push(cx.waker().clone());
        });
        core::task::Poll::Pending
    }
}

pub fn wake_sleepers(current_tick: u64) {
    let mut queue = SLEEP_QUEUE.lock();

    let expired: Vec<u64> = queue.range(..=current_tick).map(|(k, _)| *k).collect();

    for key in expired {
        if let Some(wakers) = queue.remove(&key) {
            for waker in wakers {
                waker.wake();
            }
        }
    }
}
