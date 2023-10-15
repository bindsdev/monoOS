//! Heap allocator

use super::{
    pmm::{get_frame_allocator, FRAME_ALLOCATOR},
    ALLOCATOR,
};
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
pub(super) fn init(mapper: &mut impl Mapper<Size4KiB>) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = VirtAddr::new((HEAP_END - 1) as u64);
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    let mut frame_allocator = get_frame_allocator();

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        unsafe {
            mapper
                .map_to(page, frame, flags, &mut *frame_allocator)?
                .flush()
        }
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START as *mut _, HEAP_SIZE);
    }

    Ok(())
}
