#![no_std]
#![no_main]

extern crate alloc;

use bootloader_api::{BootInfo, entry_point};
use kernel::{
    init, memory,
    qemu::{QemuExitCode, exit_qemu},
};
use log::{error, info};
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{Mapper, Page, PageTableFlags, PhysFrame, Size4KiB},
};

entry_point!(main, config = &kernel::BOOTLOADER_CONFIG);

fn main(boot_info: &'static mut BootInfo) -> ! {
    init();

    info!("Running kernel memory mapping test!");

    let mut memory = kernel::memory::init(boot_info);
    test_mapping(&mut memory);
    test_simple_allocation();

    info!("Test passed!");
    exit_qemu(QemuExitCode::Success);
}

#[panic_handler]
pub fn test_panic_handler(info: &core::panic::PanicInfo) -> ! {
    error!("Test failed!");
    error!("Error: {}", info);
    exit_qemu(QemuExitCode::Failed);
}

fn test_mapping(memory: &mut memory::MemoryContext) {
    let page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(0xdeadbeaf000));

    let frame_addr = memory
        .frame_allocator
        .allocate_frame()
        .expect("no frames available");

    let phys_frame = PhysFrame::containing_address(PhysAddr::new(frame_addr));

    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    unsafe {
        memory
            .mapper
            .map_to(page, phys_frame, flags, &mut memory.frame_allocator)
            .expect("map_to failed")
            .flush();
    }

    info!("Mapped test page.");

    let ptr = 0xdeadbeaf000 as *mut u64;

    let example_constant = 0xCAFEBABECAFEBABE;

    unsafe {
        *ptr = example_constant;
    }

    info!("Wrote value.");

    let value = unsafe { *ptr };

    info!("Read value: {:#x}", value);

    assert_eq!(value, example_constant);

    info!("Successfully verified mapped page.");
}

fn test_simple_allocation() {
    let mut vec = alloc::vec::Vec::new();
    for i in 0..1000 {
        vec.push(i);
    }

    info!("Vec length: {}", vec.len());
    assert_eq!(vec.get(999), Some(999).as_ref());

    info!("Successfully allocated on the heap!");
}
