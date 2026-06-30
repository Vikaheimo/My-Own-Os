#![no_std]
#![no_main]

use bootloader_api::{BootInfo, entry_point};
use kernel::{
    init,
    qemu::{QemuExitCode, exit_qemu},
};
use log::{error, info};

entry_point!(main, config = &kernel::BOOTLOADER_CONFIG);

#[allow(unreachable_code)]
fn main(_boot_info: &'static mut BootInfo) -> ! {
    init();

    info!("Running kernel panic test...");

    panic!("Example panic!");

    error!("Test failed!");
    exit_qemu(QemuExitCode::Success);
}

#[panic_handler]
pub fn test_panic_handler(_info: &core::panic::PanicInfo) -> ! {
    info!("Test passed!");
    exit_qemu(QemuExitCode::Success);
}
