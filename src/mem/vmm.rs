use super::{paging, pmm, PhysToVirt};
use spin::{Mutex, MutexGuard, Once};
use x86_64::{
    registers::control::Cr3,
    structures::paging::{mapper::MapToError, FrameAllocator, PageTable, PhysFrame, Size4KiB},
    VirtAddr,
};

/// A virtual address space, containing the root level 4 page table.
struct VAddressSpace {
    l4_page_table: PhysFrame,
}

impl VAddressSpace {
    /// Allocate a new virtual address space.
    fn new() -> Result<Self, MapToError<Size4KiB>> {
        let l4_page_table = {
            let frame = pmm::get_frame_allocator()
                .allocate_frame()
                .ok_or(MapToError::FrameAllocationFailed)?;

            let vaddr = frame.start_address().to_virt();

            let page_table: *mut PageTable = vaddr.as_mut_ptr();
            // SAFETY: `page_table` is not null.
            let page_table = unsafe { &mut *page_table };

            let active_l4_page_table = paging::active_l4_page_table();

            // Zero out the page table entries from the range 0..256.
            for i in 0..256 {
                page_table[i].set_unused();
            }

            // Map the higher half of the kernel's address space into this address
            // space.
            for i in 256..512 {
                page_table[i] = active_l4_page_table[i].clone();
            }

            frame
        };

        Ok(Self { l4_page_table })
    }

    /// Return the active virtual address space.
    fn active() -> Self {
        let l4_page_table = {
            let (active_l4_page_table, _) = Cr3::read();
            let pt_addr = active_l4_page_table.start_address();

            PhysFrame::containing_address(pt_addr)
        };

        Self { l4_page_table }
    }
}

struct VMallocObject {
    base: VirtAddr,
    length: usize,
}

pub(super) struct VMalloc {
    /// The address space managed by this VMM instance.
    address_space: VAddressSpace,
}

impl VMalloc {
    fn new() -> Self {
        Self {
            address_space: VAddressSpace::active(),
        }
    }
}

static VMALLOC: Once<Mutex<VMalloc>> = Once::new();

pub(super) fn get_vmalloc() -> MutexGuard<'static, VMalloc> {
    VMALLOC.get().expect("vmalloc not initialized ").lock()
}

/// Initialize the virtual memory manager.
pub(super) fn init() {
    VMALLOC.call_once(|| Mutex::new(VMalloc::new()));
}
