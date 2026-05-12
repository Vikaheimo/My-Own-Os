#![no_std]
#![no_main]

pub mod serial;

pub fn init() {
    serial::init();
}

pub mod prelude {
    pub use crate::{serial_print, serial_println};
}
