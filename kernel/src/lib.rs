#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::missing_safety_doc)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(arithmetic_overflow)]
#![deny(clippy::checked_conversions)]
#![deny(clippy::cast_possible_truncation)]
#![deny(clippy::cast_sign_loss)]
#![deny(clippy::cast_possible_wrap)]
#![deny(clippy::transmute_ptr_to_ref)]
#![deny(unnecessary_transmutes)]
#![deny(clippy::uninit_vec)]
#![deny(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![deny(clippy::panicking_unwrap)]
#![warn(clippy::indexing_slicing)]

extern crate alloc;

pub mod acpi;
pub mod apic;
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

impl TryFrom<&'static mut bootloader_api::BootInfo> for BootInfo {
    fn try_from(value: &'static mut bootloader_api::BootInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            memory_regions: &value.memory_regions,
            physical_offset: value
                .physical_memory_offset
                .take()
                .ok_or("No physical memory offset found")?,
            frame_buffer: value.framebuffer.take().ok_or("No framebuffer found")?,
        })
    }

    type Error = &'static str;
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
    let acpi = acpi::init(boot_info.physical_offset);

    apic::lapic::init(boot_info.physical_offset);

    x86_64::instructions::interrupts::enable();

    apic::lapic::calibrate();

    time::init();
    graphics::init(boot_info.frame_buffer);

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
