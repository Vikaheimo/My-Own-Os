#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![warn(clippy::missing_safety_doc)]
#![warn(clippy::undocumented_unsafe_blocks)]

extern crate alloc;

pub mod gdt;
pub mod interrupt;
pub mod logger;
pub mod memory;
pub mod qemu;
pub mod serial;

pub fn init() {
    serial::init();
    logger::init();
    gdt::init();
    interrupt::init();
}

pub const BOOTLOADER_CONFIG: bootloader_api::BootloaderConfig = {
    let mut config = bootloader_api::BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(bootloader_api::config::Mapping::Dynamic);
    config
};

pub mod prelude {
    pub use crate::{serial_print, serial_println};
}
