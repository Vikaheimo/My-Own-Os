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
}

impl AllocationHeader {
    unsafe fn write_header(data_start: u64, block_start: *mut FreeBlock) {
        log::trace!(
            "Writing allocation header at ox{:x}. Block starts at: 0x{:x}",
            data_start,
            block_start as u64
        );
        let header_start = data_start - size_of::<AllocationHeader>() as u64;
        let ptr = header_start as *mut AllocationHeader;
        unsafe { (*ptr).block_start = block_start }
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

const MIN_ALLOCATION_SIZE: u64 = 1;

unsafe impl KernelAllocator for FreeListAllocator {
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        log::debug!("New allocation: {:?}", layout);
        unsafe {
            self.init();
        }

        let mut current = self.first;
        let mut previous: *mut FreeBlock = null_mut();

        while !current.is_null() {
            let current_block = unsafe { *current };

            let block_start = current as u64;
            let block_size = current_block.size;
            let block_end = block_start + block_size;

            let header_size = size_of::<AllocationHeader>() as u64;
            let data_start = block_start + header_size;

            let aligned_start = align_up(data_start, layout.align() as u64);

            let padding = aligned_start - data_start;
            let required = header_size + padding + layout.size() as u64;

            let block_is_too_small = required > block_size;
            if block_is_too_small {
                previous = current;
                current = current_block.next;
                continue;
            }

            unsafe {
                AllocationHeader::write_header(data_start, current);
            }

            let data_end = block_start + required;
            let split_block_size = block_end - data_end;
            let required_size = header_size + MIN_ALLOCATION_SIZE;
            let should_split = split_block_size > required_size;

            if !should_split {
                log::trace!(
                    "Cannot split block. Block Size: {}, required: {}",
                    split_block_size,
                    required_size
                );

                if previous.is_null() {
                    self.first = current_block.next;
                } else {
                    unsafe {
                        (*previous).next = current_block.next;
                    }
                }

                return aligned_start as *mut u8;
            }
            log::trace!(
                "Block needs to be split. Block Size: {}, required: {}",
                split_block_size,
                required_size
            );

            let split_block_start = data_end;
            let split_block = split_block_start as *mut FreeBlock;

            log::trace!("New split block at 0x{:x}", split_block_start);

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

            return aligned_start as *mut u8;
        }
        log::error!("Allocation failed: No valid allocation location found!");

        // No space for an allocation
        null_mut()
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        if ptr.is_null() {
            log::error!("Tried to deallocate a null ptr!");
            return;
        }
        let data_start = ptr as u64;

        log::debug!("De-allocation at 0x{:x}: {:?}", data_start, layout);
        let header = unsafe { AllocationHeader::read_header(data_start) };
        let block_size = data_start + layout.size() as u64 - header.block_start as u64;
        log::trace!(
            "New free block starts at 0x{:x}. Block size: 0x{:x}",
            header.block_start as u64,
            block_size
        );

        unsafe {
            (*header.block_start).next = self.first as *mut FreeBlock;
            (*header.block_start).size = block_size
        }

        self.first = header.block_start;
    }
}
