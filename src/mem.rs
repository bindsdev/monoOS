mod paging;

use core::sync::atomic::AtomicU64;
use limine::{MemmapEntry, NonNullPtr};

pub static PHYSICAL_MEMORY_OFFSET: AtomicU64 = AtomicU64::new(0);

/// Initialize memory allocation facilities.
pub fn init(memmap: &'static mut [NonNullPtr<MemmapEntry>]) {
    paging::init(memmap);
    log::info!("initialized paging");
}
