mod heap;
mod paging;
mod pmm;

use core::sync::atomic::AtomicU64;
use limine::{MemmapEntry, NonNullPtr};
use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub static PHYSICAL_MEMORY_OFFSET: AtomicU64 = AtomicU64::new(0);

/// Initialize memory allocation facilities.
pub fn init(memmap: &'static mut [NonNullPtr<MemmapEntry>]) {
    pmm::init(memmap);
    log::info!("initialized physical memory manager");

    let mut mapper = paging::init();
    log::info!("initialized paging");

    heap::init(&mut mapper).expect("heap: initialization failed");
}
