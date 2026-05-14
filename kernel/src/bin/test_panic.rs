#![no_std]
#![no_main]

use bootloader_api::{BootInfo, entry_point};
use kernel::{
    init,
    qemu::{QemuExitCode, exit_qemu},
    serial_println,
};

entry_point!(main);

#[allow(unreachable_code)]
fn main(_boot_info: &'static mut BootInfo) -> ! {
    init();

    serial_println!("Running kernel panic test...");

    panic!("Example panic!");

    serial_println!("Test failed!");
    exit_qemu(QemuExitCode::Success);
}

#[panic_handler]
pub fn test_panic_handler(_info: &core::panic::PanicInfo) -> ! {
    serial_println!("Test passed!");
    exit_qemu(QemuExitCode::Success);
}
