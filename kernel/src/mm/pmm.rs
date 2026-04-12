use limine::request::MemmapRespData;
use limine::memmap;
use spin::Mutex;
use super::frame::{PhysFrame, FrameAllocator, FRAME_SIZE};
use crate::serial_println;

/// Global physical memory manager.
pub static FRAME_ALLOCATOR: Mutex<Option<BitmapAllocator>> = Mutex::new(None);

/// Bitmap-based physical frame allocator.
/// Each bit represents one 4 KiB frame. 0 = free, 1 = used.
pub struct BitmapAllocator {
    bitmap: &'static mut [u8],
    total_frames: usize,
    free_count: usize,
    next_free_hint: usize,
}

impl BitmapAllocator {
    fn new(bitmap: &'static mut [u8], total_frames: usize) -> Self {
        // Mark all frames as used initially
        for byte in bitmap.iter_mut() {
            *byte = 0xFF;
        }
        Self {
            bitmap,
            total_frames,
            free_count: 0,
            next_free_hint: 0,
        }
    }

    fn mark_free(&mut self, frame_index: usize) {
        if frame_index < self.total_frames {
            let byte = frame_index / 8;
            let bit = frame_index % 8;
            if self.bitmap[byte] & (1 << bit) != 0 {
                self.bitmap[byte] &= !(1 << bit);
                self.free_count += 1;
            }
        }
    }

    fn mark_used(&mut self, frame_index: usize) {
        if frame_index < self.total_frames {
            let byte = frame_index / 8;
            let bit = frame_index % 8;
            if self.bitmap[byte] & (1 << bit) == 0 {
                self.bitmap[byte] |= 1 << bit;
                self.free_count -= 1;
            }
        }
    }

    fn is_free(&self, frame_index: usize) -> bool {
        let byte = frame_index / 8;
        let bit = frame_index % 8;
        self.bitmap[byte] & (1 << bit) == 0
    }
}

impl FrameAllocator for BitmapAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        for i in 0..self.total_frames {
            let index = (self.next_free_hint + i) % self.total_frames;
            if self.is_free(index) {
                self.mark_used(index);
                self.next_free_hint = (index + 1) % self.total_frames;
                return Some(PhysFrame::from_index(index));
            }
        }
        None
    }

    fn deallocate_frame(&mut self, frame: PhysFrame) {
        let index = frame.index();
        self.mark_free(index);
        if index < self.next_free_hint {
            self.next_free_hint = index;
        }
    }

    fn free_frames(&self) -> usize {
        self.free_count
    }

    fn total_frames(&self) -> usize {
        self.total_frames
    }
}

/// Initialize the physical memory manager from the Limine memory map.
pub fn init(mmap: &MemmapRespData, hhdm_offset: u64) {
    let entries = mmap.entries();

    // Find the highest physical address to determine bitmap size
    let mut max_addr: u64 = 0;
    let mut usable_bytes: u64 = 0;
    for entry in entries.iter() {
        let end = entry.base + entry.length;
        if end > max_addr {
            max_addr = end;
        }
        if entry.type_ == memmap::MEMMAP_USABLE {
            usable_bytes += entry.length;
        }
    }

    let total_frames = (max_addr / FRAME_SIZE) as usize;
    let bitmap_bytes = (total_frames + 7) / 8;

    serial_println!(
        "[pmm] Usable memory: {} MiB, address space: {} MiB, bitmap: {} KiB",
        usable_bytes / 1024 / 1024,
        max_addr / 1024 / 1024,
        bitmap_bytes / 1024
    );

    // Find a usable region large enough for the bitmap
    let mut bitmap_phys_addr: Option<u64> = None;
    for entry in entries.iter() {
        if entry.type_ == memmap::MEMMAP_USABLE && entry.length >= bitmap_bytes as u64 {
            bitmap_phys_addr = Some(entry.base);
            break;
        }
    }

    let bitmap_phys = bitmap_phys_addr.expect("No usable region large enough for frame bitmap");

    // Map the bitmap via HHDM
    let bitmap_virt = (bitmap_phys + hhdm_offset) as *mut u8;
    let bitmap = unsafe { core::slice::from_raw_parts_mut(bitmap_virt, bitmap_bytes) };

    let mut allocator = BitmapAllocator::new(bitmap, total_frames);

    // Mark usable regions as free
    for entry in entries.iter() {
        if entry.type_ == memmap::MEMMAP_USABLE {
            let start_frame = (entry.base / FRAME_SIZE) as usize;
            let end_frame = ((entry.base + entry.length) / FRAME_SIZE) as usize;
            for frame_idx in start_frame..end_frame {
                allocator.mark_free(frame_idx);
            }
        }
    }

    // Mark the bitmap's own memory as used
    let bitmap_start_frame = (bitmap_phys / FRAME_SIZE) as usize;
    let bitmap_end_frame = ((bitmap_phys + bitmap_bytes as u64 + FRAME_SIZE - 1) / FRAME_SIZE) as usize;
    for frame_idx in bitmap_start_frame..bitmap_end_frame {
        allocator.mark_used(frame_idx);
    }

    serial_println!(
        "[pmm] Free frames: {} ({} MiB)",
        allocator.free_frames(),
        allocator.free_frames() as u64 * FRAME_SIZE / 1024 / 1024
    );

    *FRAME_ALLOCATOR.lock() = Some(allocator);
}

/// Allocate a single physical frame.
pub fn alloc_frame() -> Option<PhysFrame> {
    FRAME_ALLOCATOR.lock().as_mut()?.allocate_frame()
}

/// Free a physical frame.
pub fn free_frame(frame: PhysFrame) {
    if let Some(alloc) = FRAME_ALLOCATOR.lock().as_mut() {
        alloc.deallocate_frame(frame);
    }
}

/// Get the number of free frames.
pub fn free_count() -> usize {
    FRAME_ALLOCATOR.lock().as_ref().map_or(0, |a| a.free_frames())
}
