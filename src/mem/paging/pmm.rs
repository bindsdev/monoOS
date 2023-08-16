use limine::{MemmapEntry, MemoryMapEntryType, NonNullPtr};
use spin::{Mutex, Once};
use x86_64::structures::paging::{PageSize, Size4KiB};

pub(super) struct FrameAllocatorInner {}

impl FrameAllocatorInner {
    const PAGE_SIZE: u64 = Size4KiB::SIZE;

    pub(super) fn new(memmap: &'static mut [NonNullPtr<MemmapEntry>]) -> Self {
        let mut filtered = memmap
            .iter()
            .filter(|e| e.typ == MemoryMapEntryType::Usable);

        // The Limine protocol guarantees that memory map entries are sorted by the base address ascending.
        let first_usable = filtered.next().unwrap();
        let last_usable = filtered.last().unwrap();

        let mem_base = first_usable.base;
        let mem_end = last_usable.base + last_usable.len;
        let usable_frames = (mem_end - mem_base) / Self::PAGE_SIZE;

        Self {}
    }
}

pub(super) struct FrameAllocator(Once<Mutex<FrameAllocatorInner>>);

impl FrameAllocator {
    /// Create an uninitialized version of the frame allocator.
    const fn uninit() -> Self {
        Self(Once::new())
    }

    /// Initialize the frame allocator.
    pub(super) fn init(&self, memmap: &'static mut [NonNullPtr<MemmapEntry>]) {
        self.0
            .call_once(|| Mutex::new(FrameAllocatorInner::new(memmap)));
    }
}

pub(super) static FRAME_ALLOCATOR: FrameAllocator = FrameAllocator::uninit();
