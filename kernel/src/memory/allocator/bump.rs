use core::ptr::null_mut;

use x86_64::align_up;

use crate::memory::allocator::KernelAllocator;

#[derive(Debug)]
pub struct BumpAllocator {
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
}

// SAFETY: This allocator only advances an internal bump pointer and returns
// pointers within its configured heap range.
unsafe impl KernelAllocator for BumpAllocator {
    unsafe fn alloc(&mut self, layout: core::alloc::Layout) -> *mut u8 {
        let alloc_start = align_up(self.next, layout.align() as u64);

        let alloc_end = match alloc_start.checked_add(layout.size() as u64) {
            Some(end) => end,
            None => return null_mut(),
        };

        if alloc_end > self.heap_end {
            return null_mut();
        }

        self.next = alloc_end;
        log::debug!("Allocated {} bytes at 0x{:?}", layout.size(), alloc_start);
        alloc_start as *mut u8
    }

    unsafe fn dealloc(&mut self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        // Skipping, as dealloc is a no-op in bump allocator
    }
}
