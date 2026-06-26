use core::{
    alloc::Layout,
    mem::{align_of, size_of},
    ptr::null_mut,
};

use x86_64::align_up;

use crate::memory::allocator::KernelAllocator;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct FreeBlock {
    /// Total size including the header
    size: u64,
    next: *mut FreeBlock,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct AllocationHeader {
    block_start: *mut FreeBlock,
    block_size: u64,
}

impl AllocationHeader {
    unsafe fn write_header(data_start: u64, block_start: *mut FreeBlock, block_size: u64) {
        let header_start = data_start - size_of::<AllocationHeader>() as u64;
        let ptr = header_start as *mut AllocationHeader;

        unsafe {
            (*ptr).block_start = block_start;
            (*ptr).block_size = block_size;
        }
    }

    unsafe fn read_header(data_start: u64) -> AllocationHeader {
        let header_start = data_start - size_of::<AllocationHeader>() as u64;
        let ptr = header_start as *const AllocationHeader;

        unsafe { *ptr }
    }
}

#[derive(Debug)]
pub struct FreeListAllocator {
    first: *mut FreeBlock,
    heap_start: u64,
    heap_end: u64,
}

impl FreeListAllocator {
    pub const fn new(heap_start: u64, heap_size: u64) -> Self {
        Self {
            first: null_mut(),
            heap_start,
            heap_end: heap_start + heap_size,
        }
    }

    unsafe fn init(&mut self) {
        if !self.first.is_null() {
            return;
        }

        let aligned_start = align_up(self.heap_start, align_of::<FreeBlock>() as u64);
        let usable_size = (self.heap_end - aligned_start) & !(align_of::<FreeBlock>() as u64 - 1);

        if usable_size < size_of::<FreeBlock>() as u64 {
            return;
        }

        let block = aligned_start as *mut FreeBlock;

        unsafe {
            (*block).size = usable_size;
            (*block).next = null_mut();
        }

        self.first = block;
    }
}

unsafe impl Send for FreeListAllocator {}

unsafe impl KernelAllocator for FreeListAllocator {
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        log::debug!("New allocation: {:?}", layout);

        unsafe {
            self.init();
        }

        let mut current = self.first;
        let mut previous: *mut FreeBlock = null_mut();
        let header_size = size_of::<AllocationHeader>() as u64;

        while !current.is_null() {
            let current_block = unsafe { *current };
            let block_start = current as u64;
            let block_size = current_block.size;
            let block_end = block_start + block_size;

            let minimum_data_start = block_start + header_size;
            let aligned_start = align_up(minimum_data_start, layout.align() as u64);
            let data_end = aligned_start + layout.size() as u64;

            let split_block_start = align_up(data_end, align_of::<FreeBlock>() as u64);

            let block_is_too_small = data_end > block_end;
            if block_is_too_small {
                previous = current;
                current = current_block.next;
                continue;
            }

            // Determine if there's enough space left over to form a valid split block
            let required_split_size = size_of::<FreeBlock>() as u64;
            let should_split = (split_block_start + required_split_size) <= block_end;

            if !should_split {
                // Consume the ENTIRE block to avoid leaking or creating unaligned fragments
                if previous.is_null() {
                    self.first = current_block.next;
                } else {
                    unsafe {
                        (*previous).next = current_block.next;
                    }
                }

                unsafe {
                    AllocationHeader::write_header(aligned_start, current, block_size);
                }

                log::debug!("Allocated {} bytes at 0x{:x}", block_size, block_start);

                return aligned_start as *mut u8;
            }

            // Perform the split cleanly on an aligned boundary
            let split_block_size = block_end - split_block_start;
            let split_block = split_block_start as *mut FreeBlock;

            unsafe {
                (*split_block).next = current_block.next;
                (*split_block).size = split_block_size;
            }

            if previous.is_null() {
                self.first = split_block;
            } else {
                unsafe {
                    (*previous).next = split_block;
                }
            }

            // The actual size consumed by this allocation block up to the split point
            let allocated_total_size = split_block_start - block_start;
            unsafe {
                AllocationHeader::write_header(aligned_start, current, allocated_total_size);
            }

            log::debug!(
                "Allocated {} bytes at 0x{:x}",
                allocated_total_size,
                block_start
            );

            return aligned_start as *mut u8;
        }

        log::error!("Failed to find a valid allocation location!");
        null_mut()
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, _layout: Layout) {
        if ptr.is_null() {
            log::error!("Trying to free a null pointer!");
            return;
        }

        let data_start = ptr as u64;
        let header = unsafe { AllocationHeader::read_header(data_start) };
        log::debug!(
            "Deallocating {} bytes at 0x{:x}",
            header.block_size,
            header.block_start as u64,
        );

        unsafe {
            (*header.block_start).next = self.first;
            (*header.block_start).size = header.block_size;
        }

        self.first = header.block_start;
    }
}
