#![no_std]
#![no_main]

use bootloader_api::{BootInfo, entry_point};
use kernel::{
    graphics::{Color, Point},
    init,
    qemu::{QemuExitCode, exit_qemu},
};
use log::{error, info};

entry_point!(main, config = &kernel::BOOTLOADER_CONFIG);

#[allow(unreachable_code)]
fn main(boot_info: &'static mut BootInfo) -> ! {
    init(boot_info.try_into().unwrap());

    info!("Running graphics test!");

    let mut framebuffer = kernel::graphics::WRITER.get().unwrap().lock();

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

    framebuffer.flush();

    exit_qemu(QemuExitCode::Success);
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("PANIC: {info}");
    exit_qemu(QemuExitCode::Failed);
}
