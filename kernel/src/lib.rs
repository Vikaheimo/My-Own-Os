#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![warn(clippy::missing_safety_doc)]
#![warn(clippy::undocumented_unsafe_blocks)]

extern crate alloc;

pub mod acpi;
pub mod asynchronous;
pub mod gdt;
pub mod graphics;
pub mod interrupt;
pub mod logger;
pub mod memory;
pub mod qemu;
pub mod serial;
pub mod time;

#[derive(Debug)]
pub struct BootInfo {
    physical_offset: u64,
    memory_regions: &'static bootloader_api::info::MemoryRegions,
    frame_buffer: bootloader_api::info::FrameBuffer,
}

impl From<&'static mut bootloader_api::BootInfo> for BootInfo {
    fn from(value: &'static mut bootloader_api::BootInfo) -> Self {
        Self {
            memory_regions: &value.memory_regions,
            physical_offset: value
                .physical_memory_offset
                .take()
                .expect("No physical memory offset found"),
            frame_buffer: value.framebuffer.take().expect("No framebuffer found"),
        }
    }
}

#[derive(Debug)]
pub struct KernelInfo {
    pub memory: memory::MemoryContext,
    pub acpi: acpi::KernelAcpiHandler,
}

pub fn init(boot_info: BootInfo) -> KernelInfo {
    serial::init();
    logger::init();
    gdt::init();
    interrupt::init();
    let memory = memory::init(boot_info.physical_offset, boot_info.memory_regions);
    graphics::init(boot_info.frame_buffer);
    let acpi = acpi::init(boot_info.physical_offset);
    time::init();

    log::info!("Kernel initialized");

    KernelInfo { memory, acpi }
}

pub const BOOTLOADER_CONFIG: bootloader_api::BootloaderConfig = {
    let mut config = bootloader_api::BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(bootloader_api::config::Mapping::Dynamic);
    config
};

pub mod prelude {
    pub use crate::{serial_print, serial_println};
}
