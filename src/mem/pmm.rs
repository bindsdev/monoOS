use limine::{MemmapEntry, MemoryMapEntryType, NonNullPtr};
use spin::{Mutex, MutexGuard, Once};
use x86_64::{
    structures::paging::{FrameAllocator, FrameDeallocator, PageSize, PhysFrame, Size4KiB},
    PhysAddr,
};

pub(super) struct SystemFrameAllocator {
    bitmap: &'static mut [u32],
    mem_size_frames: u64,
    next_free_bit: usize,
}

impl SystemFrameAllocator {
    const FRAME_SIZE: u64 = Size4KiB::SIZE;

    pub(super) fn new(memmap: &'static mut [NonNullPtr<MemmapEntry>]) -> Self {
        let last_entry = memmap.last().unwrap();
        let mem_size_bytes = last_entry.base + last_entry.len;
        let mem_size_frames = mem_size_bytes / Self::FRAME_SIZE;

        let bitmap_size = mem_size_frames.div_ceil(8);
        let mut bitmap_region_base_ptr: *mut u32 = core::ptr::null_mut();

        for usable_region in memmap
            .iter()
            .filter(|e| e.typ == MemoryMapEntryType::Usable)
        {
            if usable_region.len >= bitmap_size {
                bitmap_region_base_ptr = usable_region.base as *mut _;
                break;
            }
        }

        if bitmap_region_base_ptr.is_null() {
            panic!("not enough free memory to store bitmap requiring a size of {bitmap_size}");
        }

        // SAFETY: `bitmap_region_base_ptr` and `bitmap_size` uphold the safety contract for `core::slice::from_raw_parts_mut`.
        let bitmap = unsafe {
            core::slice::from_raw_parts_mut::<'static>(bitmap_region_base_ptr, bitmap_size as usize)
        };

        let mut allocator = Self {
            bitmap,
            mem_size_frames,
            next_free_bit: 0,
        };

        allocator.clear_bitmap();

        // Limine never marks memory from 0x0 to 0x1000 as usable.
        allocator.mark_frame(0, true);

        // Mark the region used by the bitmap as used.
        let bitmap_region_base_addr = bitmap_region_base_ptr as u64;
        allocator.mark_contiguous_frames(
            bitmap_region_base_addr,
            bitmap_region_base_addr + bitmap_size,
            true,
        );

        // Mark the unusable regions as used.
        for unusable_region in memmap
            .iter()
            .filter(|e| e.typ != MemoryMapEntryType::Usable)
        {
            let base = unusable_region.base;
            let end = base + unusable_region.len;

            allocator.mark_contiguous_frames(base, end, true);
        }

        let chunk = allocator
            .bitmap
            .iter()
            .position(|c| *c != u32::MAX)
            .expect("pmm: no memory remaining");
        let trailing_ones = allocator.bitmap[chunk].trailing_ones() as usize; // This OS runs on a little-endian architecture, so the most significant bits are stored last.
        let first_free_bit = chunk * 32 + trailing_ones;
        allocator.next_free_bit = first_free_bit;

        allocator
    }

    /// Clear the bitmap.
    fn clear_bitmap(&mut self) {
        for i in 0..self.bitmap.len() {
            self.bitmap[i] = u32::MIN;
        }
    }

    /// Mark a frame with the status indicated by `allocated`.
    fn mark_frame(&mut self, frame: usize, allocated: bool) {
        let chunk = frame / 32;
        let chunk_bit = frame % 32;

        if allocated {
            self.bitmap[chunk] |= 1 << chunk_bit;
            self.next_free_bit = frame + 1;
        } else {
            self.bitmap[chunk] &= !(1 << chunk_bit);
            self.next_free_bit = frame - 1;
        }
    }

    /// Mark the next free frame as allocated.
    fn mark_next_frame(&mut self) {
        self.mark_frame(self.next_free_bit, true);
    }

    /// Mark a contiguous chunk of frames starting at physical address `base` and ending at
    /// physical address `end` exclusively with the status indicated by `allocated`.
    fn mark_contiguous_frames(&mut self, base: u64, end: u64, allocated: bool) {
        let base = base / Self::FRAME_SIZE;
        let end = end.div_ceil(Self::FRAME_SIZE);

        for frame in base..end {
            self.mark_frame(frame as usize, allocated);
        }
    }

    /// Check if a frame is used.
    fn is_frame_used(&mut self, frame: usize) -> bool {
        let chunk = frame / 32;
        let chunk_bit = frame % 32;

        (self.bitmap[chunk] & (1 << chunk_bit)) != 0
    }

    /// Allocate a frame.
    fn alloc_frame(&mut self) -> Option<PhysAddr> {
        let bit = self.next_free_bit;
        self.mark_next_frame();

        Some(PhysAddr::new(bit as u64 * Self::FRAME_SIZE))
    }

    /// Deallocate the frame starting at `frame_addr`.
    fn dealloc_frame(&mut self, frame_addr: PhysAddr) {
        let frame = frame_addr.as_u64() / Self::FRAME_SIZE;
        self.mark_frame(frame as usize, false);
    }
}

unsafe impl FrameAllocator<Size4KiB> for SystemFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let phys = self.alloc_frame()?;
        Some(PhysFrame::containing_address(phys))
    }
}

impl FrameDeallocator<Size4KiB> for SystemFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        let phys = frame.start_address();
        self.dealloc_frame(phys);
    }
}

pub(super) static FRAME_ALLOCATOR: Once<Mutex<SystemFrameAllocator>> = Once::new();

pub(super) fn get_frame_allocator() -> MutexGuard<'static, SystemFrameAllocator> {
    FRAME_ALLOCATOR.get().unwrap().lock()
}

/// Initialize the physical memory manager.
pub(super) fn init(memmap: &'static mut [NonNullPtr<MemmapEntry>]) {
    FRAME_ALLOCATOR.call_once(|| Mutex::new(SystemFrameAllocator::new(memmap)));
}
