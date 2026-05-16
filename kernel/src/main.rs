#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use bootloader_api::{BootInfo, entry_point};
use kernel::{
    BOOTLOADER_CONFIG,
    prelude::*,
    qemu::{QemuExitCode, exit_qemu},
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    kernel::init();
    serial_println!("Entered kernel with boot info: {boot_info:?}");
    exit_qemu(QemuExitCode::Success);
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    serial_println!("PANIC: {info}");
    exit_qemu(QemuExitCode::Failed);
}
