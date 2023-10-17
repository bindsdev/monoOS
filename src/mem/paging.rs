use super::{physical_memory_offset, PhysToVirt};
use x86_64::{
    registers::control::Cr3,
    structures::paging::{OffsetPageTable, PageTable},
    VirtAddr,
};

pub(super) fn active_l4_page_table() -> &'static mut PageTable {
    let (l4_page_table, _) = Cr3::read();

    let vaddr = l4_page_table.start_address().to_virt();

    let l4_page_table = vaddr.as_mut_ptr();

    // SAFETY: `l4_page_table` is not null.
    unsafe { &mut *l4_page_table }
}

pub(super) fn mapper() -> OffsetPageTable<'static> {
    let active_l4_page_table = active_l4_page_table();

    // SAFETY: The reference to the active level 4 page table is correct and the correct physical memory offset is provided.
    unsafe {
        OffsetPageTable::new(
            active_l4_page_table,
            VirtAddr::new(physical_memory_offset()),
        )
    }
}
