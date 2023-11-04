//! Bootstrap allocator

use core::{
    alloc::{AllocError, Allocator, Layout},
    ptr::NonNull,
    slice,
};
use x86_64::{
    structures::paging::{FrameAllocator, FrameDeallocator, PhysFrame},
    VirtAddr,
};

use super::{pmm, PhysToVirt};

pub(super) struct BootstrapAllocator;

// SAFETY:
unsafe impl Allocator for BootstrapAllocator {
    #[track_caller]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        assert_eq!(
            layout.align(),
            4096,
            "bootstrap allocator must allocate 4 KiB blocks"
        );

        let frame = pmm::get_pmm().allocate_frame().ok_or(AllocError)?;
        let vaddr = frame.start_address().to_virt();

        // SAFETY: the pointer is valid for 4096 bytes.
        Ok(unsafe {
            NonNull::new_unchecked(slice::from_raw_parts_mut(vaddr.as_mut_ptr().cast(), 4096))
        })
    }

    #[track_caller]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        assert_eq!(
            layout.align(),
            4096,
            "bootstrap allocator must allocate 4 KiB blocks"
        );

        let vaddr = VirtAddr::from_ptr(ptr.as_ptr() as *const _);
        let paddr = todo!();
        // SAFETY: the given start address is properly aligned.
        let frame = unsafe { PhysFrame::from_start_address_unchecked(paddr) };

        // SAFETY: the frame is unused.
        unsafe { pmm::get_pmm().deallocate_frame(frame) };
    }
}
