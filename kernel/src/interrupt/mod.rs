use super::prelude::*;
use log::info;
use spin::Once;
use x86_64::{
    instructions::port::Port,
    registers::control::Cr2,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
};

mod pic;

pub const LAPIC_SPURIOUS_HANDLER_VECTOR: u8 = 0xFF;

#[cfg(feature = "interrupts-pit")]
pub const PIT_FREQUENCY_HZ: u32 = 1000;

#[cfg(feature = "interrupts-pit")]
pub static PIT_TICS: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

static IDT: Once<InterruptDescriptorTable> = Once::new();

pub fn init() {
    let idt = IDT.call_once(|| {
        let mut idt = InterruptDescriptorTable::new();

        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        // SAFETY: The configured stack index points to a valid IST entry
        // initialized in the TSS during early boot.
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX as u16);
        }
        idt.general_protection_fault
            .set_handler_fn(general_protection_fault);
        idt.divide_error.set_handler_fn(divide_by_zero_handler);

        idt[LAPIC_SPURIOUS_HANDLER_VECTOR].set_handler_fn(lapic_spurious_handler);

        #[cfg(feature = "interrupts-pit")]
        idt[pic::InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);

        idt
    });

    idt.load();
    info!("IDT loaded");

    // SAFETY: PIC initialization performs required hardware port I/O during
    // early kernel setup, before normal interrupt handling begins.
    #[cfg(feature = "interrupts-pit")]
    unsafe {
        pic::init_pics();
    }

    #[cfg(feature = "interrupts-pit")]
    init_pit(PIT_FREQUENCY_HZ);
    x86_64::instructions::interrupts::enable();

    info!("PIC enabled")
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    serial_println!("EXCEPTION: Breakpoint");
    serial_println!("{:#?}", stack_frame)
}

extern "x86-interrupt" fn divide_by_zero_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DIVIDE BY ZERO\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n {:#?}", stack_frame)
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    panic!(
        "EXCEPTION: PAGE FAULT\nAccessed Address: {:?}\nError Code: {:?}\n{:#?}",
        Cr2::read(),
        error_code,
        stack_frame
    )
}

extern "x86-interrupt" fn general_protection_fault(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "EXCEPTION: GENERAL PROTECTION FAULT\nError Code: {:#x}\n{:#?}",
        error_code, stack_frame
    )
}

extern "x86-interrupt" fn lapic_spurious_handler(stack_frame: InterruptStackFrame) {
    log::warn!(
        "Spurious interrupt: RIP={:#x}, CS={:#x}, RFLAGS={:#x}",
        stack_frame.instruction_pointer.as_u64(),
        stack_frame.code_segment.0,
        stack_frame.cpu_flags.bits(),
    );
}

#[cfg(feature = "interrupts-pit")]
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use core::sync::atomic::Ordering;

    let tick = PIT_TICS.fetch_add(1, Ordering::Relaxed);

    crate::asynchronous::sleep::wake_sleepers(tick);

    // SAFETY: This is called from the timer interrupt handler.
    // The interrupt index corresponds to a valid hardware IRQ,
    // and access is synchronized via the Mutex.
    unsafe {
        pic::PICS
            .lock()
            .notify_end_of_interrupt(pic::InterruptIndex::Timer.as_u8());
    }
}

/// Initializes the Programmable Interval Timer (PIT) to generate
/// periodic interrupts at the given frequency (in Hz).
///
/// This programs PIT channel 0 in mode 3 (square wave generator)
/// and configures it to trigger IRQ0 at `frequency` times per second.
///
/// # Arguments
///
/// * `frequency` - The desired interrupt frequency in Hertz (Hz).
///
/// For example:
/// * `100`  → 100 interrupts per second (10 ms per tick)
/// * `1000` → 1000 interrupts per second (1 ms per tick)
///
/// # Details
///
/// The PIT runs at a base frequency of 1,193,182 Hz. The divisor
/// is computed as:
///
/// ```text
/// divisor = 1_193_182 / frequency
/// ```
///
/// The divisor must fit in 16 bits (≤ 65535), which limits the
/// minimum achievable frequency to approximately 18.2 Hz.
///
/// # Safety
///
/// This function performs raw I/O port writes to hardware ports
/// `0x43` (command) and `0x40` (channel 0 data), which is required
/// to configure the PIT.
pub fn init_pit(frequency: u32) {
    let divisor: u16 = (1_193_182 / frequency) as u16;

    let mut command = Port::<u8>::new(0x43);
    let mut channel0 = Port::<u8>::new(0x40);

    // SAFETY: Writing PIT command/data bytes to these fixed I/O ports is
    // required to program channel 0 with the computed divisor.
    unsafe {
        command.write(0x36);
        channel0.write((divisor & 0xFF) as u8);
        channel0.write((divisor >> 8) as u8);
    }
}
