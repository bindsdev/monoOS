use limine::{MemmapEntry, MemoryMapEntryType, NonNullPtr};
use spin::{Mutex, Once};
use x86_64::{
    structures::paging::{FrameAllocator, FrameDeallocator, PageSize, PhysFrame, Size4KiB},
    PhysAddr,
};

pub(super) struct FrameAllocatorInner {
    bitmap: &'static mut [u32],
    mem_size_frames: u64,
}

impl FrameAllocatorInner {
    const FRAME_SIZE: u64 = Size4KiB::SIZE;

    pub(super) fn new(memmap: &'static mut [NonNullPtr<MemmapEntry>]) -> Self {
        let last_entry = memmap.last().unwrap();
        let mem_size_bytes = last_entry.base + last_entry.len;
        let mem_size_frames = mem_size_bytes / Self::FRAME_SIZE;

        let bitmap_size = mem_size_frames.div_ceil(8);
        let mut bitmap_region_base: *mut u32 = core::ptr::null_mut();

        for usable_region in memmap
            .iter()
            .filter(|e| e.typ == MemoryMapEntryType::Usable)
        {
            if usable_region.len >= bitmap_size {
                bitmap_region_base = usable_region.base as *mut _;
                break;
            }
        }

        if bitmap_region_base.is_null() {
            panic!("not enough free memory to store bitmap requiring a size of {bitmap_size}");
        }

        // SAFETY: `bitmap_region_base` and `bitmap_size` uphold the safety contract for `core::slice::from_raw_parts_mut`.
        let bitmap = unsafe {
            core::slice::from_raw_parts_mut::<'static>(bitmap_region_base, bitmap_size as usize)
        };

        let mut allocator = Self {
            bitmap,
            mem_size_frames,
        };

        allocator.clear_bitmap();

        // Mark the region used by the bitmap as used.
        // SAFETY: `bitmap_region_base` upholds the safety contract for `core::ptr::read`.
        let bitmap_addr = unsafe { bitmap_region_base.read() } as u64;
        allocator.mark_contiguous_frames(bitmap_addr, bitmap_addr + bitmap_size, true);

        // Mark the unusable regions as used.
        for unusable_region in memmap
            .iter()
            .filter(|e| e.typ != MemoryMapEntryType::Usable)
        {
            let base = unusable_region.base;
            let end = base + unusable_region.len;

            allocator.mark_contiguous_frames(base, end, true);
        }

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
        } else {
            self.bitmap[chunk] &= !(1 << chunk_bit);
        }
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
    fn allocate_frame_inner(&mut self) -> Option<PhysAddr> {
        // Find the first free bit.
        let chunk = self.bitmap.iter().position(|c| *c != u32::MAX)?;
        let trailing_ones = self.bitmap[chunk].trailing_ones() as usize;
        let frame = chunk * 32 + trailing_ones;

        if frame >= self.mem_size_frames as usize {
            return None;
        }

        self.mark_frame(frame, true);

        Some(PhysAddr::new(frame as u64 * Self::FRAME_SIZE))
    }

    /// Deallocate the frame starting at `frame_addr`.
    fn deallocate_frame_inner(&mut self, frame_addr: PhysAddr) {
        self.mark_frame(frame_addr.as_u64() as usize, false);
    }
}

pub(super) struct SystemFrameAllocator(Once<Mutex<FrameAllocatorInner>>);

impl SystemFrameAllocator {
    /// Create an uninitialized version of the frame allocator.
    const fn uninit() -> Self {
        Self(Once::new())
    }

    /// Initialize the frame allocator.
    pub(super) fn init(&self, memmap: &'static mut [NonNullPtr<MemmapEntry>]) {
        self.0
            .call_once(|| Mutex::new(FrameAllocatorInner::new(memmap)));
    }

    /// Allocate a frame.
    pub(super) fn allocate_frame(&mut self) -> Option<PhysAddr> {
        self.0.get().unwrap().lock().allocate_frame_inner()
    }

    /// Deallocate the frame starting at `frame_addr`.
    pub(super) fn deallocate_frame(&mut self, frame_addr: PhysAddr) {
        self.0
            .get()
            .unwrap()
            .lock()
            .deallocate_frame_inner(frame_addr);
    }
}

unsafe impl FrameAllocator<Size4KiB> for SystemFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let phys = self.allocate_frame()?;
        Some(PhysFrame::containing_address(phys))
    }
}

impl FrameDeallocator<Size4KiB> for SystemFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        let phys = frame.start_address();
        self.deallocate_frame(phys);
    }
}

pub(super) static FRAME_ALLOCATOR: SystemFrameAllocator = SystemFrameAllocator::uninit();
