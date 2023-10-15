use super::{pmm::*, PHYSICAL_MEMORY_OFFSET};
use core::sync::atomic::Ordering;
use limine::{MemmapEntry, NonNullPtr};
use x86_64::{
    registers::control::Cr3,
    structures::paging::{mapper::MapToError, OffsetPageTable, PageTable, Size4KiB},
    VirtAddr,
};

fn active_l4_page_table() -> &'static mut PageTable {
    let (l4_page_table, _) = Cr3::read();

    let virt_addr = VirtAddr::new(
        l4_page_table.start_address().as_u64() + PHYSICAL_MEMORY_OFFSET.load(Ordering::Relaxed),
    );

    let l4_page_table = virt_addr.as_mut_ptr();

    unsafe { &mut *l4_page_table }
}

// Initialize paging.
pub(super) fn init() -> OffsetPageTable<'static> {
    let l4_page_table = active_l4_page_table();

    // SAFETY: the reference to the level 4 page table is valid and the physical offset passed is correct.
    let offset_page_table = unsafe {
        OffsetPageTable::new(
            l4_page_table,
            VirtAddr::new(PHYSICAL_MEMORY_OFFSET.load(Ordering::Relaxed)),
        )
    };

    offset_page_table
}
