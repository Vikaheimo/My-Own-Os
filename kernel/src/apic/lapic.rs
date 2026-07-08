use core::sync::atomic::{AtomicU32, Ordering};
use core::time::Duration;

use spin::Once;
use x86_64::registers::model_specific::{ApicBase, ApicBaseFlags};

const REG_EOI: usize = 0xB0;
const REG_SPURIOUS: usize = 0xF0;

const REG_LVT_TIMER: usize = 0x320;
const REG_INITIAL_COUNT: usize = 0x380;
const REG_CURRENT_COUNT: usize = 0x390;
const REG_DIVIDE: usize = 0x3E0;

const INITIAL_COUNT: u32 = 0xFFFF_FFFF;

const DIVIDE_BY_1: u32 = 0b1011;

const PERIODIC_MODE: u32 = 1 << 17;

const LAPIC_TIMER_FREQUENCY: u64 = 1000;

pub static LAPIC: Once<Lapic> = Once::new();

pub fn init(physical_offset: u64) {
    let (frame, mut flags) = ApicBase::read();

    flags.insert(ApicBaseFlags::LAPIC_ENABLE);

    // SAFETY: We are enabling the LAPIC in IA32_APIC_BASE using values read
    // from the same MSR, preserving the base frame while setting enable flags.
    unsafe {
        ApicBase::write(frame, flags);
    }

    let lapic = Lapic::new(physical_offset + frame.start_address().as_u64());

    lapic.enable(crate::interrupt::LAPIC_SPURIOUS_HANDLER_VECTOR);

    LAPIC.call_once(|| lapic);

    log::info!("LAPIC enabled");
}

pub fn calibrate() {
    const CALIBRATION_DURATION_MS: u64 = 100;
    const CALIBRATION_DURATION: Duration = Duration::from_millis(CALIBRATION_DURATION_MS);
    const PIT_TICKS_WAITED: u64 =
        crate::interrupt::PIT_FREQUENCY_HZ as u64 * CALIBRATION_DURATION_MS / 1000;

    let lapic = LAPIC.get().unwrap();

    let current_pit_tick = crate::interrupt::PIT_TICS.load(Ordering::Acquire);
    lapic.start_calibration();

    let end_tick = current_pit_tick + PIT_TICKS_WAITED;
    while crate::interrupt::PIT_TICS.load(Ordering::Relaxed) < end_tick {
        core::hint::spin_loop();
    }

    lapic.finish_calibration(CALIBRATION_DURATION);
    crate::time::MONOTONIC_TICK_FREQUENCY.store(LAPIC_TIMER_FREQUENCY, Ordering::Relaxed);
    lapic.start_periodic(crate::time::frequency_to_period(LAPIC_TIMER_FREQUENCY));
}

pub struct Lapic {
    base: *mut u32,
    ticks_per_ms: AtomicU32,
}

// SAFETY: `Lapic` accesses MMIO through volatile reads/writes, and all shared
// access goes through methods that do not create aliasing references.
unsafe impl Send for Lapic {}
// SAFETY: Register accesses are side-effectful MMIO operations performed with
// volatile primitives, which are safe to call from shared references.
unsafe impl Sync for Lapic {}

impl Lapic {
    pub fn new(base: u64) -> Self {
        Self {
            base: base as *mut u32,
            ticks_per_ms: AtomicU32::new(0),
        }
    }

    fn write(&self, offset: usize, value: u32) {
        // SAFETY: `self.base` points to the mapped LAPIC MMIO page and offsets
        // used by callers are LAPIC register offsets aligned to 32-bit words.
        unsafe {
            core::ptr::write_volatile(self.base.add(offset / 4), value);
        }
    }

    fn read(&self, offset: usize) -> u32 {
        // SAFETY: `self.base` points to the mapped LAPIC MMIO page and offsets
        // used by callers are LAPIC register offsets aligned to 32-bit words.
        unsafe { core::ptr::read_volatile(self.base.add(offset / 4)) }
    }

    pub fn enable(&self, spurious_vector: u8) {
        self.write(REG_SPURIOUS, 0x100 | spurious_vector as u32);
    }

    pub fn eoi(&self) {
        self.write(REG_EOI, 0);
    }

    /// Starts the LAPIC timer in one-shot mode.
    pub fn start_calibration(&self) {
        self.write(REG_DIVIDE, DIVIDE_BY_1);

        // one-shot mode
        self.write(REG_LVT_TIMER, crate::interrupt::LAPIC_TIMER_VECTOR as u32);

        self.write(REG_INITIAL_COUNT, INITIAL_COUNT);
    }

    /// Finishes calibration.
    ///
    /// Call this after waiting with the PIT.
    pub fn finish_calibration(&self, elapsed: Duration) {
        let current = self.read(REG_CURRENT_COUNT);

        let elapsed_ticks = INITIAL_COUNT - current;
        let frequency = elapsed_ticks / elapsed.as_millis() as u32;
        self.ticks_per_ms.store(frequency, Ordering::Release);

        log::info!("LAPIC calibrated: {} ticks/ms", frequency);
    }

    pub fn ticks_per_ms(&self) -> u32 {
        self.ticks_per_ms.load(Ordering::Acquire)
    }

    pub fn start_periodic(&self, period: Duration) {
        let ticks = self.ticks_per_ms() * period.as_millis() as u32;

        self.write(REG_DIVIDE, DIVIDE_BY_1);

        self.write(
            REG_LVT_TIMER,
            crate::interrupt::LAPIC_TIMER_VECTOR as u32 | PERIODIC_MODE,
        );

        self.write(REG_INITIAL_COUNT, ticks);
    }

    pub fn stop(&self) {
        self.write(REG_INITIAL_COUNT, 0);
    }

    pub fn current_count(&self) -> u32 {
        self.read(REG_CURRENT_COUNT)
    }
}
