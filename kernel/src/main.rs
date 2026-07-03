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
    kernel::init();
    info!("Entered kernel with boot info: {boot_info:?}");

    let BootInfo {
        physical_memory_offset,
        memory_regions,
        framebuffer,
        ..
    } = boot_info;
    let physical_offset = physical_memory_offset
        .into_option()
        .expect("physical memory not mapped");

    let _memory = kernel::memory::init(physical_offset, memory_regions);
    info!("Memory initialized.");

    let mut framebuffer = kernel::graphics::FramebufferWriter::new(framebuffer.take().unwrap());

    framebuffer.clear_screen(kernel::graphics::Color {
        red: 0,
        green: 0,
        blue: 0,
    });

    info!("Screen cleared");

    framebuffer.set_pixel(
        kernel::graphics::Point { x: 500, y: 500 },
        kernel::graphics::Color {
            red: 255,
            green: 0,
            blue: 0,
        },
    );

    let white = kernel::graphics::Color {
        red: 255,
        green: 255,
        blue: 255,
    };

    framebuffer.draw_string(
        "Hello world!",
        kernel::graphics::Point { x: 0, y: 2 },
        white,
    );

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
