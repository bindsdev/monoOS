//! Heap allocator

use super::{pmm, vmm, PhysToVirt, ALLOCATOR};
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

const HEAP_SIZE: usize = 100 * 1024; // 100 KiB
const HEAP_START: usize = 0x444444440000;
const HEAP_END: usize = HEAP_START + HEAP_SIZE;

/// Initialize heap.
pub(super) fn init() -> Result<(), MapToError<Size4KiB>> {
    vmm::get_vmalloc().allocate(
        HEAP_START + (HEAP_END - 1),
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
    );

    unsafe {
        ALLOCATOR.lock().init(HEAP_START as *mut _, HEAP_SIZE);
    }

    Ok(())
}
