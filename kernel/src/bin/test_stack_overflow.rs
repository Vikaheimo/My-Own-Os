#![no_std]
#![no_main]

use bootloader_api::{BootInfo, entry_point};
use kernel::{
    init,
    qemu::{QemuExitCode, exit_qemu},
};
use log::{error, info};

entry_point!(main, config = &kernel::BOOTLOADER_CONFIG);

fn main(boot_info: &'static mut BootInfo) -> ! {
    init(boot_info.into());

    info!("Running kernel stack overflow test...");

    overflow();

    error!("This should not happen!");
    exit_qemu(QemuExitCode::Failed);
}

#[allow(unconditional_recursion)]
fn overflow() {
    overflow();
}

#[panic_handler]
pub fn test_panic_handler(_info: &core::panic::PanicInfo) -> ! {
    info!("Test passed!");
    exit_qemu(QemuExitCode::Success)
}
