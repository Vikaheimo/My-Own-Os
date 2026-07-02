#![no_std]
#![no_main]

use bootloader_api::{BootInfo, entry_point};
use kernel::{
    graphics::{Color, FramebufferWriter, Point},
    init,
    qemu::{QemuExitCode, exit_qemu},
};
use log::{error, info};

entry_point!(main, config = &kernel::BOOTLOADER_CONFIG);

#[allow(unreachable_code)]
fn main(boot_info: &'static mut BootInfo) -> ! {
    init();
    let BootInfo {
        physical_memory_offset,
        memory_regions,
        framebuffer,
        ..
    } = boot_info;

    let physical_offset = physical_memory_offset
        .into_option()
        .expect("physical memory not mapped");

    let mut _memory = kernel::memory::init(physical_offset, memory_regions);

    info!("Running graphics test!");

    let mut framebuffer = FramebufferWriter::new(framebuffer.take().unwrap());

    framebuffer.clear_screen(Color {
        red: 235,
        green: 52,
        blue: 119,
    });

    framebuffer.draw_string(
        "Some really important string!",
        Point { x: 10, y: 10 },
        Color {
            red: 123,
            green: 123,
            blue: 123,
        },
    );

    exit_qemu(QemuExitCode::Success);
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("PANIC: {info}");
    exit_qemu(QemuExitCode::Failed);
}
