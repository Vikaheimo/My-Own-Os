use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::null_mut,
};
use spin::Mutex;
use x86_64::{
    VirtAddr, align_up,
    structures::paging::{
        FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB, mapper::MapToError,
    },
};

#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator::new();

pub const HEAP_START: u64 = 0x_4444_4444_0000;
pub const HEAP_SIZE: u64 = 100 * 1024;

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let start_page = Page::containing_address(heap_start);
        let end_page = Page::containing_address(heap_end);
        Page::range_inclusive(start_page, end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;

        let flags: PageTableFlags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }

    Ok(())
}

#[derive(Debug)]
pub struct Allocator {
    allocator: Mutex<BumpAllocator>,
}

impl Allocator {
    pub const fn new() -> Self {
        Allocator {
            allocator: Mutex::new(BumpAllocator::new(HEAP_START, HEAP_SIZE)),
        }
    }
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe { self.allocator.lock().alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        unsafe { self.allocator.lock().dealloc(ptr, layout) }
    }
}

#[derive(Debug)]
struct BumpAllocator {
    heap_end: u64,
    next: u64,
}

impl BumpAllocator {
    pub const fn new(heap_start: u64, heap_size: u64) -> Self {
        Self {
            heap_end: heap_start + heap_size,
            next: heap_start,
        }
    }

    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let alloc_start = align_up(self.next, layout.align() as u64);

        let alloc_end = match alloc_start.checked_add(layout.size() as u64) {
            Some(end) => end,
            None => return null_mut(),
        };

        if alloc_end > self.heap_end {
            return null_mut();
        }

        self.next = alloc_end;
        alloc_start as *mut u8
    }

    unsafe fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout) {}
}
