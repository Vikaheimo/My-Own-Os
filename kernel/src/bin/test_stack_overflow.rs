#![no_std]
#![no_main]

use bootloader_api::{BootInfo, entry_point};
use kernel::{
    init,
    qemu::{QemuExitCode, exit_qemu},
    serial_println,
};

entry_point!(main);

fn main(_boot_info: &'static mut BootInfo) -> ! {
    init();

    serial_println!("Running kernel stack overflow test...");

    overflow();

    serial_println!("Test passed!");
    exit_qemu(QemuExitCode::Success);
}

#[allow(unconditional_recursion)]
fn overflow() {
    overflow();
}

#[panic_handler]
pub fn test_panic_handler(info: &core::panic::PanicInfo) -> ! {
    serial_println!("[failed]");
    serial_println!("Error: {}", info);
    exit_qemu(QemuExitCode::Failed);
}
