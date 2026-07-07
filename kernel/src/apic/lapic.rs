use spin::Once;
use x86_64::registers::model_specific::{ApicBase, ApicBaseFlags};

pub static LAPIC: Once<Lapic> = Once::new();

pub fn init(physical_offset: u64) {
    let (frame, mut flags) = ApicBase::read();
    flags.insert(ApicBaseFlags::LAPIC_ENABLE);
    unsafe { ApicBase::write(frame, flags) };

    let lapic_phys = frame.start_address().as_u64();
    let lapic_virt = physical_offset + lapic_phys;

    let lapic = Lapic::new(lapic_virt);

    lapic.lapic_software_enable();
    lapic.init_timer();
    
    LAPIC.call_once(|| lapic);

    log::info!("Lapic initialized");
}

#[derive(Debug, Clone, Copy)]
pub struct Lapic {
    base: *mut u32,
}

unsafe impl Send for Lapic {}
unsafe impl Sync for Lapic {}

impl Lapic {
    pub fn new(virtual_base: u64) -> Self {
        Self {
            base: virtual_base as *mut u32,
        }
    }

    pub fn lapic_software_enable(&self) {
        const SPURIOUS: usize = 0xF0;

        // Bit 8 = APIC software enable
        // Lower 8 bits = spurious interrupt vector (must exist in IDT)
        let vector = crate::interrupt::LAPIC_SPURIOUS_HANDLER_VECTOR as u32 & 0xFF;
        let value = 0x100 | vector;

        self.write(SPURIOUS, value);
    }

    pub fn init_timer(&self) {
        const TIMER_LVT: usize = 0x320;
        const TIMER_DIVIDE: usize = 0x3E0;
        const TIMER_INITIAL_COUNT: usize = 0x380;

        self.write(TIMER_DIVIDE, 0b0011);

        // Bit 17 = periodic mode
        let lvt_value = (crate::interrupt::LAPIC_TIMER_VECTOR as u32) | (1 << 17); // periodic

        self.write(TIMER_LVT, lvt_value);

        // 3️⃣ Set initial count (trial value)
        self.write(TIMER_INITIAL_COUNT, 10_000_000);
    }

    #[inline]
    fn write(&self, offset: usize, value: u32) {
        unsafe {
            core::ptr::write_volatile(self.base.add(offset / 4), value);
        }
    }

    #[inline]
    fn read(&self, offset: usize) -> u32 {
        unsafe { core::ptr::read_volatile(self.base.add(offset / 4)) }
    }

    pub fn eoi(&self) {
        const EOI: usize = 0xB0;
        self.write(EOI, 0);
    }
}
