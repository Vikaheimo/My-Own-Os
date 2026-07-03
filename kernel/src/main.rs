#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

extern crate alloc;

use core::time::Duration;

use bootloader_api::{BootInfo, entry_point};
use kernel::{
    BOOTLOADER_CONFIG,
    asynchronous::{executor::AsyncExecutor, sleep::Sleep, task::Task},
    graphics::{Color, FramebufferWriter},
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

    let framebuffer = FramebufferWriter::new(framebuffer.take().unwrap());

    let mut executor = AsyncExecutor::new();
    executor.spawn(Task::new(spinning_cube(framebuffer)));

    executor.run();
}

const BLACK: Color = Color {
    red: 0,
    green: 0,
    blue: 0,
};

const WHITE: Color = Color {
    red: 255,
    green: 255,
    blue: 255,
};

async fn spinning_cube(mut framebuffer: FramebufferWriter) {
    let mut angle: f32 = 0.0;

    loop {
        let sleep = Sleep::new(Duration::from_millis(0));
        framebuffer.clear_screen(BLACK);
        framebuffer.draw_cube(angle, WHITE);
        framebuffer.flush();
        angle += 0.03;

        sleep.await;
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("PANIC: {info}");
    exit_qemu(QemuExitCode::Failed);
}
