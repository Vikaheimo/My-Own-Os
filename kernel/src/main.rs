#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

extern crate alloc;

use bootloader_api::{BootInfo, entry_point};
use kernel::{
    BOOTLOADER_CONFIG,
    qemu::{QemuExitCode, exit_qemu},
};
use log::{error, info};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    kernel::init();
    info!("Entered kernel with boot info: {boot_info:?}");

    let BootInfo {
        physical_memory_offset,
        memory_regions,
        ..
    } = boot_info;
    let physical_offset = physical_memory_offset
        .into_option()
        .expect("physical memory not mapped");

    let _memory = kernel::memory::init(physical_offset, memory_regions);
    info!("Memory initialized.");

    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("PANIC: {info}");
    exit_qemu(QemuExitCode::Failed);
}
