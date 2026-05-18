use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use x86_64::{
    PhysAddr,
    structures::paging::{FrameAllocator, PhysFrame, Size4KiB},
};

const FRAME_SIZE: u64 = 4096;
/**
We want to skip the first 1MiB for legacy reasons and debugging
 */
const MIN_USABLE_ADDR: u64 = 0x100000;

#[derive(Debug)]
pub struct BootInfoFrameAllocator {
    memory_regions: &'static MemoryRegions,
    physical_offset: u64,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Creates a new `BootInfoFrameAllocator` from the bootloader's
    /// memory map.
    ///
    /// The allocator will return physical frames marked as
    /// `MemoryRegionKind::Usable`, skipping the first 1 MiB of memory.
    ///
    /// # Assumptions
    ///
    /// - `memory_regions` must accurately describe the system's physical
    ///   memory layout.
    /// - All regions marked as `Usable` must not overlap with memory used
    ///   by the kernel, bootloader, or hardware.
    /// - `physical_offset` must correctly map physical memory into the
    ///   virtual address space so that frames can be zeroed safely.
    ///
    /// Violating these assumptions may lead to undefined behavior during
    /// frame allocation.
    pub fn init(memory_regions: &'static MemoryRegions, physical_offset: u64) -> Self {
        Self {
            memory_regions,
            physical_offset,
            next: 0,
        }
    }

    /// Returns an iterator over all usable physical frames.
    ///
    /// The iterator:
    ///
    /// - Yields only frames from regions marked as
    ///   `MemoryRegionKind::Usable`.
    /// - Skips all memory below `MIN_USABLE_ADDR` (1 MiB).
    /// - Aligns region boundaries to 4 KiB (`FRAME_SIZE`).
    /// - Produces frame start addresses in ascending order.
    ///
    /// Each yielded `u64` represents the starting physical address
    /// of a 4 KiB-aligned frame.
    ///
    /// # Notes
    ///
    /// - The correctness of this iterator depends on the bootloader's
    ///   memory map accurately describing usable memory.
    /// - The iterator is recreated on each call and walks the memory
    ///   regions from the beginning.
    fn usable_frames(&self) -> impl Iterator<Item = u64> + '_ {
        self.memory_regions
            .iter()
            .filter(|r| r.kind == MemoryRegionKind::Usable)
            .flat_map(|r| {
                let start = align_up(r.start, FRAME_SIZE).max(MIN_USABLE_ADDR);
                let end = align_down(r.end, FRAME_SIZE);

                (start..end).step_by(FRAME_SIZE as usize)
            })
    }

    /// Allocates the next available physical frame.
    ///
    /// Returns the starting physical address of a 4 KiB frame,
    /// or `None` if no usable frames remain.
    ///
    /// The allocated frame:
    ///
    /// - Comes from a region marked as `MemoryRegionKind::Usable`.
    /// - Is 4 KiB aligned.
    /// - Has not been previously returned by this allocator.
    /// - Is zeroed before being handed out.
    ///
    /// # Notes
    ///
    /// This function assumes that physical memory is mapped at
    /// `self.physical_offset`, so that the frame can be safely
    /// zeroed through the corresponding virtual address.
    ///
    /// If the physical memory mapping is incorrect, zeroing the
    /// frame may cause undefined behavior.
    pub fn allocate_frame(&mut self) -> Option<u64> {
        let frame = self.usable_frames().nth(self.next)?;
        self.next += 1;

        let virtual_address = frame + self.physical_offset;
        unsafe {
            // SAFETY: The frame lies in a region marked as usable and
            // physical memory is mapped at `self.physical_offset`.
            core::ptr::write_bytes(virtual_address as *mut u8, 0, FRAME_SIZE as usize);
        }

        Some(frame)
    }
}

/// # Safety
///
/// This implementation guarantees that:
///
/// - Each returned frame is aligned to 4 KiB.
/// - Each frame lies within a memory region marked as
///   `MemoryRegionKind::Usable`.
/// - No frame is returned more than once.
/// - Frames below 1 MiB are never returned.
/// - Frames are zeroed before being handed out.
///
/// The correctness of this implementation depends on the
/// bootloader-provided memory map being accurate and not marking
/// reserved or in-use memory as `Usable`.
unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<x86_64::structures::paging::PhysFrame<Size4KiB>> {
        let frame_address = self.allocate_frame()?;
        Some(PhysFrame::containing_address(PhysAddr::new(frame_address)))
    }
}

fn align_up(address: u64, align: u64) -> u64 {
    (address + align - 1) & !(align - 1)
}

fn align_down(address: u64, align: u64) -> u64 {
    address & !(align - 1)
}
