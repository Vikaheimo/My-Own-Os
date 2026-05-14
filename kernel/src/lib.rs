#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

pub mod gdt;
pub mod interrupt;
pub mod qemu;
pub mod serial;

pub fn init() {
    serial::init();
    gdt::init();
    interrupt::init();
}

pub mod prelude {
    pub use crate::{serial_print, serial_println};
}
