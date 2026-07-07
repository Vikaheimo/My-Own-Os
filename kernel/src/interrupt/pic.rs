use pic8259::ChainedPics;
use spin::Mutex;

// SAFETY: PIC_1_OFFSET and PIC_2_OFFSET are valid, non-overlapping
// interrupt vector offsets. This is called only once during static
// initialization before interrupts are enabled.
pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

const IRQ_TIMER: u8 = 0;

const PIC_MASTER_MASK_ALL: u8 = 0xFF;
const PIC_SLAVE_MASK_ALL: u8 = 0xFF;

const PIC_MASTER_MASK_TIMER_ONLY: u8 = PIC_MASTER_MASK_ALL & !(1 << IRQ_TIMER);

pub unsafe fn init_pics() {
    let mut pics = PICS.lock();
    // SAFETY: We are in early kernel initialization before interrupts
    // are enabled. Access to the PIC is synchronized through the Mutex,
    // and no concurrent access can occur at this stage.
    unsafe {
        pics.initialize();
        pics.write_masks(PIC_MASTER_MASK_TIMER_ONLY, PIC_SLAVE_MASK_ALL);
    }
}

pub unsafe fn disable_pic() {
    use x86_64::instructions::port::Port;

    let mut port1 = Port::<u8>::new(0x21);
    let mut port2 = Port::<u8>::new(0xA1);
    unsafe {
        port1.write(PIC_MASTER_MASK_ALL);
        port2.write(PIC_SLAVE_MASK_ALL);
    }
}
