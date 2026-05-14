#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

pub mod interrupt;
pub mod qemu;
pub mod serial;

pub fn init() {
    interrupt::init();
    serial::init();
}


pub mod prelude {
    pub use crate::{serial_print, serial_println};
}
