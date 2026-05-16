#![no_std]
#![no_main]

use bootloader_api::{BootInfo, entry_point};
use kernel::{
    init,
    qemu::{QemuExitCode, exit_qemu},
    serial_println,
};

entry_point!(main, config = &kernel::BOOTLOADER_CONFIG);

fn main(_boot_info: &'static mut BootInfo) -> ! {
    init();

    serial_println!("Running kernel stack overflow test...");

    overflow();

    serial_println!("This should not happen!");
    exit_qemu(QemuExitCode::Failed);
}

#[allow(unconditional_recursion)]
fn overflow() {
    overflow();
}

#[panic_handler]
pub fn test_panic_handler(_info: &core::panic::PanicInfo) -> ! {
    serial_println!("Test passed!");
    exit_qemu(QemuExitCode::Success)
}
