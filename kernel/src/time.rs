const TSC_FREQUENCY: u64 = 1_000_000_000;
const NS_PER_SEC: u128 = 1_000_000_000;

pub fn uptime_ns() -> u64 {
    ((rdtsc() as u128 * NS_PER_SEC) / TSC_FREQUENCY as u128) as u64
}

const NS_PER_MS: u64 = 1_000_000;

pub fn uptime_ms() -> u64 {
    uptime_ns() / NS_PER_MS
}

#[inline]
pub fn rdtsc() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() }
}
