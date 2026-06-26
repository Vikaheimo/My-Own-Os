#![no_std]
#![no_main]

extern crate alloc;

use core::alloc::GlobalAlloc;

use bootloader_api::{BootInfo, entry_point};
use kernel::{
    init,
    memory::{self, heap::GLOBAL_HEAP_ALLOCATOR},
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
    test_unaligned_split_corruption();

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

/// This test should be verified by hand!
/// Check that the allocations are correctly aligned!
/// TODO: Automate this test
pub fn test_unaligned_split_corruption() {
    use core::alloc::Layout;
    info!("Testing unaligned allocation!");

    let layout_odd = Layout::from_size_align(5, 1).unwrap();

    let ptr1 = unsafe { GLOBAL_HEAP_ALLOCATOR.alloc(layout_odd) };
    assert!(!ptr1.is_null(), "First allocation failed");

    let layout_next = Layout::from_size_align(8, 8).unwrap();
    let ptr2 = unsafe { GLOBAL_HEAP_ALLOCATOR.alloc(layout_next) };
    assert!(
        !ptr2.is_null(),
        "Second allocation failed due to header corruption"
    );

    // Clean up
    unsafe {
        GLOBAL_HEAP_ALLOCATOR.dealloc(ptr1, layout_odd);
        GLOBAL_HEAP_ALLOCATOR.dealloc(ptr2, layout_next);
    }

    info!("Successfully allocated on the heap!")
}
