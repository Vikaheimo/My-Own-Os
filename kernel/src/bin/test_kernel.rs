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

    serial_println!("Running kernel tests...");

    test_basic();
    test_breakpoint();

    serial_println!("All tests passed!");
    exit_qemu(QemuExitCode::Success);
}

#[allow(clippy::eq_op)]
fn test_basic() {
    assert_eq!(1, 1);
}

fn test_breakpoint() {
    x86_64::instructions::interrupts::int3();
}

#[panic_handler]
pub fn test_panic_handler(info: &core::panic::PanicInfo) -> ! {
    serial_println!("[failed]");
    serial_println!("Error: {}", info);
    exit_qemu(QemuExitCode::Failed);
}
