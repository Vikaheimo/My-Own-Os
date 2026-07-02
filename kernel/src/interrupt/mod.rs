use super::prelude::*;
use core::sync::atomic::{AtomicU64, Ordering};
use log::info;
use spin::Once;
use x86_64::{
    instructions::port::Port,
    registers::control::Cr2,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
};

mod pic;

static TIMER_INTERRUPT_TICS: AtomicU64 = AtomicU64::new(0);

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

        idt[pic::InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);

        idt
    });

    idt.load();
    info!("IDT loaded");

    // SAFETY: PIC initialization performs required hardware port I/O during
    // early kernel setup, before normal interrupt handling begins.
    unsafe {
        pic::init_pics();
    }

    init_pit(100);
    x86_64::instructions::interrupts::enable();

    info!("PIC enabled")
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    serial_println!("EXCEPTION: Breakpoint");
    serial_println!("{:#?}", stack_frame)
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

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    TIMER_INTERRUPT_TICS.fetch_add(1, Ordering::Relaxed);

    // SAFETY: This is called from the timer interrupt handler.
    // The interrupt index corresponds to a valid hardware IRQ,
    // and access is synchronized via the Mutex.
    unsafe {
        pic::PICS
            .lock()
            .notify_end_of_interrupt(pic::InterruptIndex::Timer.as_u8());
    }
}

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
