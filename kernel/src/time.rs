use core::sync::atomic::{AtomicU64, Ordering};

static TSC_FREQUENCY: AtomicU64 = AtomicU64::new(0);

/// Number of nanoseconds in one second.
/// Stored as u128 to prevent overflow during intermediate multiplication.
const NS_PER_SEC: u128 = 1_000_000_000;

/// Number of nanoseconds in one millisecond.
const NS_PER_MS: u64 = 1_000_000;

pub fn calibrate_tsc() {
    const CALIBRATION_TICKS: u64 = 100;

    // Wait for next tick edge (optional but improves precision)
    let start_tick = crate::interrupt::TIMER_INTERRUPT_TICS.load(Ordering::Relaxed);

    while crate::interrupt::TIMER_INTERRUPT_TICS.load(Ordering::Relaxed) == start_tick {}

    let start_tsc = rdtsc();
    let target_tick = start_tick + CALIBRATION_TICKS;

    while crate::interrupt::TIMER_INTERRUPT_TICS.load(Ordering::Relaxed) < target_tick {}

    let end_tsc = rdtsc();
    let delta_tsc = end_tsc - start_tsc;

    // Convert measured ticks into seconds using PIT_FREQUENCY_HZ
    //
    // elapsed_seconds = CALIBRATION_TICKS / PIT_FREQUENCY_HZ
    //
    // frequency = delta_tsc / elapsed_seconds
    //           = delta_tsc * PIT_FREQUENCY_HZ / CALIBRATION_TICKS
    let tsc_frequency = (delta_tsc * crate::interrupt::PIT_FREQUENCY_HZ as u64) / CALIBRATION_TICKS;
    TSC_FREQUENCY.store(tsc_frequency, Ordering::Relaxed);
    log::info!("Calibrated TSC frequency: {} Hz", tsc_frequency);
}

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
    let freq = TSC_FREQUENCY.load(Ordering::Relaxed);

    // Safety check (avoid divide by zero during early boot)
    if freq == 0 {
        return 0;
    }

    ((rdtsc() as u128 * NS_PER_SEC) / freq as u128) as u64
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
