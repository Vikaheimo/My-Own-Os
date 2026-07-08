#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

extern crate alloc;

use core::time::Duration;

use bootloader_api::{BootInfo, entry_point};
use kernel::{
    BOOTLOADER_CONFIG,
    asynchronous::{executor::AsyncExecutor, sleep::Sleep, task::Task},
    qemu::{QemuExitCode, exit_qemu},
};
use log::{error, info};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    kernel::init(boot_info.try_into().unwrap());

    let mut executor = AsyncExecutor::new();
    executor.spawn(Task::new(logging_task()));

    executor.run();
}

async fn logging_task() {
    loop {
        info!("Log!");
        Sleep::new(Duration::from_secs(1)).await;
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("PANIC: {info}");
    exit_qemu(QemuExitCode::Failed);
}
