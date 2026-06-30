/// Assumed TSC frequency in Hz (ticks per second).
///
/// IMPORTANT:
/// This must match your CPU's actual Time Stamp Counter frequency.
/// On modern systems with invariant TSC, this is usually the base
/// (non‑turbo) frequency. If this value is incorrect, all time
/// calculations will be wrong.
const TSC_FREQUENCY: u64 = 1_000_000_000;

/// Number of nanoseconds in one second.
/// Stored as u128 to prevent overflow during intermediate multiplication.
const NS_PER_SEC: u128 = 1_000_000_000;

/// Number of nanoseconds in one millisecond.
const NS_PER_MS: u64 = 1_000_000;

/// Returns system uptime in nanoseconds.
///
/// This works by:
/// 1. Reading the CPU Time Stamp Counter (TSC) via `rdtsc()`.
/// 2. Converting ticks to nanoseconds using:
///    time_ns = ticks * (1e9 / TSC_FREQUENCY)
///
/// We perform the multiplication in u128 to avoid overflow:
///   (ticks * 1_000_000_000) could overflow u64 otherwise.
pub fn uptime_ns() -> u64 {
    ((rdtsc() as u128 * NS_PER_SEC) / TSC_FREQUENCY as u128) as u64
}

/// Returns system uptime in milliseconds.
///
/// This simply converts the nanosecond uptime into milliseconds.
pub fn uptime_ms() -> u64 {
    uptime_ns() / NS_PER_MS
}

/// Reads the CPU Time Stamp Counter (TSC).
///
/// The TSC is a 64-bit register that increments every CPU cycle.
/// On modern x86_64 systems with invariant TSC, it increments at a
/// constant rate independent of CPU frequency scaling.
///
#[inline]
pub fn rdtsc() -> u64 {
    // Safety:
    // `_rdtsc()` is unsafe because it directly emits a CPU instruction.
    // It is safe to call on x86_64 hardware that supports TSC.
    unsafe { core::arch::x86_64::_rdtsc() }
}
