mod frame_allocator;
mod heap;
mod paging;

pub struct MemoryContext {
    pub mapper: x86_64::structures::paging::OffsetPageTable<'static>,
    pub frame_allocator: frame_allocator::BootInfoFrameAllocator,
}

/// Initializes the memory subsystem and returns a new `MemoryContext`.
///
/// This function:
/// - Initializes a frame allocator using the memory map provided by the
///   bootloader.
/// - Creates an `OffsetPageTable` for the currently active level 4 page
///   table using the physical memory offset supplied by the bootloader.
///
/// # Panics
///
/// Panics if the bootloader did not provide a physical memory offset.
///
/// # Safety
///
/// This function is safe to call because it relies on the guarantees
/// provided by the bootloader:
///
/// - The physical memory is correctly mapped at the given
///   `physical_memory_offset`.
/// - The memory regions in `boot_info.memory_regions` accurately describe
///   usable and reserved memory.
/// - This function is called only once during early kernel initialization.
///
/// If these guarantees are violated, undefined behavior may occur in the
/// paging or frame allocation code.
pub fn init(boot_info: &'static bootloader_api::BootInfo) -> MemoryContext {
    let phys_offset = boot_info
        .physical_memory_offset
        .into_option()
        .expect("physical memory not mapped");

    let mut frame_allocator =
        frame_allocator::BootInfoFrameAllocator::init(&boot_info.memory_regions, phys_offset);

    // SAFETY: The bootloader provides a valid physical memory offset mapping,
    // and memory initialization runs once during early kernel boot.
    let mut mapper = unsafe { paging::init_offset_page_table(phys_offset) };

    heap::init_heap(&mut mapper, &mut frame_allocator).unwrap();

    MemoryContext {
        mapper,
        frame_allocator,
    }
}
