use core::sync::atomic::{AtomicU64, Ordering};

/// Global storage for the detected TSC frequency in Hz.
///
/// This is initialized once during early boot via [`init`]
/// and then read by uptime calculation functions.
///
/// Stored as `AtomicU64` to allow safe concurrent reads
/// without locking.
static TSC_FREQUENCY: AtomicU64 = AtomicU64::new(0);

/// Number of nanoseconds in one second.
///
/// Stored as `u128` to prevent overflow during intermediate
/// multiplication when converting TSC cycles to nanoseconds.
const NS_PER_SEC: u128 = 1_000_000_000;

/// Initializes the global TSC frequency.
///
/// This attempts to determine the TSC frequency using CPUID
/// leaf `0x15`. If that is unavailable or returns invalid data,
/// it falls back to calibrating the TSC against the PIT timer.
///
/// This function must be called once during early boot
/// before calling [`uptime_ns`] or [`uptime_ms`].
pub fn init() {
    let freq = tsc_frequency_from_cpuid().unwrap_or_else(calibrate_pit);

    TSC_FREQUENCY.store(freq, Ordering::Relaxed);
}

#[cfg(not(feature = "interrupts-pit"))]
#[inline]
pub fn calibrate_pit() -> u64 {
    unimplemented!()
}


/// Calibrates the TSC frequency using the PIT (Programmable Interval Timer).
///
/// This measures the number of TSC cycles elapsed over a fixed
/// number of PIT timer ticks and computes the TSC frequency from that.
///
/// This is slower than using CPUID but works on systems where
/// CPUID leaf `0x15` is unavailable or not implemented.
///
/// # Returns
///
/// The measured TSC frequency in Hertz (cycles per second).
///
/// # Notes
///
/// - Assumes the PIT interrupt counter is running.
/// - Assumes interrupts are enabled.
/// - Blocks until the calibration period completes.
#[cfg(feature = "interrupts-pit")]
pub fn calibrate_pit() -> u64 {
    const CALIBRATION_TICKS: u64 = 100;

    let start_tick = crate::interrupt::PIT_TICS.load(Ordering::Relaxed);

    // Wait for the next PIT tick so we start on a clean boundary.
    while crate::interrupt::PIT_TICS.load(Ordering::Relaxed) == start_tick {
        core::hint::spin_loop();
    }

    let start_tsc = rdtsc();
    let target_tick = start_tick + CALIBRATION_TICKS;

    while crate::interrupt::PIT_TICS.load(Ordering::Relaxed) < target_tick {
        core::hint::spin_loop();
    }

    let end_tsc = rdtsc();
    let delta_tsc = end_tsc - start_tsc;

    let tsc_frequency = (delta_tsc * crate::interrupt::PIT_FREQUENCY_HZ as u64) / CALIBRATION_TICKS;

    log::info!("Calibrated TSC frequency using PIT: {} Hz", tsc_frequency);

    tsc_frequency
}

/// Attempts to determine the TSC frequency using CPUID leaf `0x15`.
///
/// CPUID leaf `0x15` (if supported) provides the ratio between
/// the TSC frequency and the core crystal clock frequency.
///
/// The formula used is:
///
/// ```text
/// TSC frequency = crystal_frequency * numerator / denominator
/// ```
///
/// # Returns
///
/// - `Some(frequency_hz)` if successfully determined.
/// - `None` if the leaf is unsupported or contains invalid data.
///
/// # Notes
///
/// - Not all CPUs implement leaf `0x15`.
/// - Some firmware implementations return zeroed values.
/// - This method is preferred when available because it is fast
///   and does not require active timing calibration.
pub fn tsc_frequency_from_cpuid() -> Option<u64> {
    let max_leaf = core::arch::x86_64::__cpuid(0).eax;

    if max_leaf < 0x15 {
        return None;
    }

    let leaf = core::arch::x86_64::__cpuid(0x15);

    let denom = leaf.eax;
    let numer = leaf.ebx;
    let crystal = leaf.ecx;

    if denom == 0 || numer == 0 || crystal == 0 {
        return None;
    }

    let freq = (crystal as u64 * numer as u64) / denom as u64;

    log::info!("TSC frequency from CPUID: {} Hz", freq);

    Some(freq)
}

/// Returns the elapsed time since system boot.
///
/// This function reads the TSC and converts it to a [`core::time::Duration`]
/// using the TSC frequency that was initialized via [`init`].
///
/// # Returns
///
/// The elapsed time as a `Duration`. If [`init`] has not been called,
/// returns `Duration::ZERO` to avoid division by zero.
///
/// # Panics
///
/// This function does not panic, but callers must ensure [`init`]
/// has been called first for accurate timing measurements.
pub fn uptime() -> core::time::Duration {
    let freq = TSC_FREQUENCY.load(Ordering::Relaxed);

    // If init() was not called, avoid division by zero.
    if freq == 0 {
        return core::time::Duration::ZERO;
    }

    let cycles = rdtsc() as u128;

    // Convert cycles to nanoseconds:
    //
    // seconds = cycles / freq
    // nanos   = (cycles * 1_000_000_000) / freq
    //
    // Use u128 to prevent overflow during multiplication.
    let nanos = (cycles * NS_PER_SEC) / freq as u128;

    core::time::Duration::from_nanos(nanos as u64)
}

/// Reads the CPU Time Stamp Counter (TSC).
///
/// The TSC is a 64-bit register that increments every CPU cycle.
/// On modern x86_64 systems with an *invariant TSC*, it increments
/// at a constant rate independent of CPU frequency scaling.
///
/// # Safety
///
/// This function executes the `RDTSC` instruction.
/// It is safe to call on x86_64 systems that support TSC.
///
/// # Notes
///
/// - This instruction is not serializing.
/// - If strict ordering is required, use `RDTSCP` or a serializing
///   instruction such as `LFENCE` before `RDTSC`.
#[inline]
pub fn rdtsc() -> u64 {
    // Safety:
    // `_rdtsc()` is unsafe because it directly emits a CPU instruction.
    // It is safe to call on x86_64 hardware that supports TSC.
    unsafe { core::arch::x86_64::_rdtsc() }
}
