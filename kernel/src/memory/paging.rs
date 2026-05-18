use x86_64::{
    VirtAddr,
    registers::control::Cr3,
    structures::paging::{OffsetPageTable, PageTable},
};

/// Initializes a new `OffsetPageTable`.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `physical_memory_offset` correctly maps the entire physical memory
///   into the virtual address space.
/// - The returned level 4 page table is the currently active page table.
/// - The mapping remains valid for the lifetime of the returned
pub(crate) unsafe fn init_offset_page_table(
    physical_memory_offset: u64,
) -> OffsetPageTable<'static> {
    let level_4_table: &mut PageTable = unsafe { active_level_4_table(physical_memory_offset) };
    unsafe { OffsetPageTable::new(level_4_table, VirtAddr::new(physical_memory_offset)) }
}

/// Returns a mutable reference to the active level 4 page table.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `physical_memory_offset` correctly maps the complete physical memory
///   into the virtual address space.
/// - The level 4 frame returned by `CR3` is valid and mapped at the
///   calculated virtual address.
/// - The returned reference is the only mutable reference to the level 4
///   page table (i.e., no aliasing `&mut` references exist).
/// - The mapping remains valid for the entire lifetime of the returned
///   reference.
unsafe fn active_level_4_table(physical_memory_offset: u64) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();

    let physical_address = level_4_table_frame.start_address().as_u64();
    let virtual_address = physical_address + physical_memory_offset;

    let page_table_ptr = virtual_address as *mut PageTable;
    unsafe { &mut *page_table_ptr }
}
