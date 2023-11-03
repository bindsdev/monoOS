mod heap;
mod paging;
mod pmm;
mod vmm;

use core::sync::atomic::{AtomicU64, Ordering};
use limine::{MemmapEntry, NonNullPtr};
use linked_list_allocator::LockedHeap;
use x86_64::{structures::paging::PageTableFlags, PhysAddr, VirtAddr};

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub static PHYSICAL_MEMORY_OFFSET: AtomicU64 = AtomicU64::new(0);

#[inline]
fn physical_memory_offset() -> u64 {
    PHYSICAL_MEMORY_OFFSET.load(Ordering::Relaxed)
}

/// Initialize memory allocation facilities.
pub fn init(memmap: &'static mut [NonNullPtr<MemmapEntry>]) {
    pmm::init(memmap);
    log::info!("initialized physical memory manager");

    vmm::init();
    log::info!("initialized virtual memory manager");

    // let addr = vmm::get_vmalloc().allocate(1, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
    // log::info!("{addr:#?}");

    // heap::init().expect("heap: initialization failed");
}

/// Convert a physical address to a virtual address.
pub trait PhysToVirt {
    fn to_virt(self) -> VirtAddr;
}

impl PhysToVirt for PhysAddr {
    fn to_virt(self) -> VirtAddr {
        VirtAddr::new(self.as_u64() + physical_memory_offset())
    }
}
