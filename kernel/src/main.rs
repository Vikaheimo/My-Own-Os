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

    let _memory = kernel::memory::init(boot_info);
    info!("Memory initialized.");

    let data = alloc::boxed::Box::new("Hello from the heap!");
    info!("Reading data from the heap: {}", data);
    info!("box: {:p}", *data);

    exit_qemu(QemuExitCode::Success);
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("PANIC: {info}");
    exit_qemu(QemuExitCode::Failed);
}
