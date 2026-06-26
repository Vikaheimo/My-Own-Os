use core::alloc::GlobalAlloc;
use spin::Mutex;

use crate::memory::heap::{HEAP_SIZE, HEAP_START};

#[cfg(feature = "memory-arena-heap")]
mod bump;
#[cfg(not(feature = "memory-arena-heap"))]
mod free_list;

#[cfg(not(feature = "memory-arena-heap"))]
#[derive(Debug)]
pub struct Allocator {
    allocator: Mutex<free_list::FreeListAllocator>,
}

#[cfg(feature = "memory-arena-heap")]
pub struct Allocator {
    allocator: Mutex<bump::BumpAllocator>,
}

impl Allocator {
    #[cfg(not(feature = "memory-arena-heap"))]
    pub const fn new() -> Self {
        Allocator {
            allocator: Mutex::new(free_list::FreeListAllocator::new(HEAP_START, HEAP_SIZE)),
        }
    }

    #[cfg(feature = "memory-arena-heap")]
    pub const fn new() -> Self {
        Allocator {
            allocator: Mutex::new(bump::BumpAllocator::new(HEAP_START, HEAP_SIZE)),
        }
    }
}

// SAFETY: Allocation and deallocation are delegated to the inner
// `KernelAllocator` implementation while synchronizing all access through
// the mutex, preserving its invariants.
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        // SAFETY: This forwards the `GlobalAlloc::alloc` contract to the
        // inner allocator while holding the mutex for exclusive access.
        unsafe { self.allocator.lock().alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        // SAFETY: This forwards the `GlobalAlloc::dealloc` contract for the
        // same `ptr`/`layout` pair to the inner allocator under mutex.
        unsafe { self.allocator.lock().dealloc(ptr, layout) }
    }
}

/// Interface for the kernel heap backend used by `Allocator`.
///
/// # Safety
/// Implementors must ensure that `alloc`/`dealloc` uphold Rust allocation
/// contracts for pointer validity, alignment, and layout matching.
pub unsafe trait KernelAllocator {
    /// # Safety
    /// The caller must uphold the allocation contract for `layout`.
    unsafe fn alloc(&mut self, layout: core::alloc::Layout) -> *mut u8;

    /// # Safety
    /// `ptr` must have been allocated by this allocator with the same `layout`
    /// and must not be used after this call.
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: core::alloc::Layout);
}
