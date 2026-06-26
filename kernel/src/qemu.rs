use log::info;
use x86_64::instructions::{hlt, port::Port};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    info!("Exiting QEMU");
    // SAFETY: Writing to I/O port 0xF4 is the documented QEMU isa-debug-exit
    // interface used to terminate the emulator with an exit code.
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }

    loop {
        hlt();
    }
}
