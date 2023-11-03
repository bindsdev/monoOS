//! Virtual memory manager implemented using a free list.

use super::{paging, pmm, PhysToVirt};
use spin::{Mutex, MutexGuard, Once};
use x86_64::{
    registers::control::Cr3,
    structures::paging::{
        mapper::{MapToError, Mapper},
        FrameAllocator, OffsetPageTable, Page, PageSize, PageTable, PageTableFlags, PhysFrame,
        Size4KiB,
    },
    VirtAddr,
};

const VMALLOC_START: usize = 0xfffff80000000000;
const VMALLOC_SIZE: usize = 128 * 1024 * 1024;

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
        let (l4_page_table, _) = Cr3::read();
        Self { l4_page_table }
    }

    /// Return the page table allocated for this virtual address space.
    fn page_table(&mut self) -> &'static mut PageTable {
        // SAFETY: `as_mut_ptr` won't return a null pointer.
        unsafe { &mut *(self.l4_page_table.start_address().to_virt().as_mut_ptr()) }
    }

    /// Return the mapper that points to the page table allocated for this virtual address space.
    fn mapper(&mut self) -> OffsetPageTable<'_> {
        // SAFETY: The reference to the active level 4 page table is correct and the correct physical memory offset is provided.
        unsafe {
            OffsetPageTable::new(
                self.page_table(),
                VirtAddr::new(super::physical_memory_offset()),
            )
        }
    }
}

/// A free region of virtual memory.
#[derive(Debug)]
struct FreeVMRegion {
    /// The base address of this free region of virtual memory.
    base: VirtAddr,

    /// The length, in bytes, of this free region of virtual memory.
    len: usize,

    prev: *mut FreeVMRegion,
    next: *mut FreeVMRegion,
}

impl FreeVMRegion {
    fn new(base: u64, len: usize) -> Self {
        Self {
            base: VirtAddr::new(base),
            len,
            prev: core::ptr::null_mut(),
            next: core::ptr::null_mut(),
        }
    }
}

pub(super) struct VMAlloc {
    inner: Mutex<VMAllocInner>,
}

impl VMAlloc {
    fn new() -> Self {
        Self {
            inner: Mutex::new(VMAllocInner::new()),
        }
    }

    /// Allocate a given amount of pages with the given flags.
    pub(super) fn allocate(&mut self, pages: usize, flags: PageTableFlags) -> Option<VirtAddr> {
        self.inner.lock().allocate(pages, flags)
    }
}

pub(super) struct VMAllocInner {
    /// The address space managed by this VMM instance.
    address_space: VAddressSpace,

    /// Doubly-linked list used to store the free portions of the virtual address space.
    free_list: *mut FreeVMRegion,
}

impl VMAllocInner {
    fn new() -> Self {
        let mut free_list = FreeVMRegion::new(VMALLOC_START as u64, VMALLOC_SIZE);
        let free_list = unsafe { &mut free_list as *mut _ };

        Self {
            address_space: VAddressSpace::active(),
            free_list,
        }
    }

    fn allocate(&mut self, pages: usize, flags: PageTableFlags) -> Option<VirtAddr> {
        let requested_bytes = pages * Size4KiB::SIZE as usize;

        let mut cur = self.free_list;
        let mut sufficient_region: Option<FreeVMRegion> = None;

        while !cur.is_null() {
            let region = unsafe { core::ptr::read(cur) };

            if region.len >= requested_bytes {
                sufficient_region = Some(region);
                break;
            }

            cur = region.next;
        }

        let sufficient_region = sufficient_region?;
        let vaddr = sufficient_region.base;
        let len = sufficient_region.len;

        // TODO: if `len` == `requested_bytes`, fully remove region from free list. else, resize it.

        let page_range = {
            let start_page = Page::containing_address(vaddr);
            let end_page = Page::containing_address(vaddr + requested_bytes);

            Page::range_inclusive(start_page, end_page)
        };

        let mut mapper = self.address_space.mapper();
        let mut frame_allocator = pmm::get_frame_allocator();

        for page in page_range {
            let frame = frame_allocator
                .allocate_frame()
                .expect("physical memory exhausted");

            unsafe {
                mapper.map_to(page, frame, flags, &mut *frame_allocator);
            }
        }

        Some(vaddr)
    }
}

// SAFETY: `VMAllocInner` is protected by a `Mutex`.
unsafe impl Send for VMAlloc {}
// SAFETY: See above.
unsafe impl Sync for VMAlloc {}

static VMALLOC: Once<VMAlloc> = Once::new();

/// Get a handle to the virtual memory manager.
///
/// # Panics
///
/// This function will panic if the virtual memory manager has not been initialized.
// pub(super) fn get_vmalloc() -> VMAlloc {
//     VMALLOC.get().expect("vmalloc not initialized ")
// }

/// Initialize the virtual memory manager.
pub(super) fn init() {
    VMALLOC.call_once(|| VMAlloc::new());
}
