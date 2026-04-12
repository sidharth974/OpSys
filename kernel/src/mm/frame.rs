use super::address::PhysAddr;

/// The size of a standard page frame (4 KiB).
pub const FRAME_SIZE: u64 = 4096;

/// Represents a physical memory frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysFrame {
    pub start: PhysAddr,
}

impl PhysFrame {
    /// Create a frame from a physical address (must be page-aligned).
    pub fn containing_address(addr: PhysAddr) -> Self {
        Self {
            start: PhysAddr::new(addr.as_u64() & !(FRAME_SIZE - 1)),
        }
    }

    pub fn from_index(index: usize) -> Self {
        Self {
            start: PhysAddr::new(index as u64 * FRAME_SIZE),
        }
    }

    pub fn index(self) -> usize {
        (self.start.as_u64() / FRAME_SIZE) as usize
    }
}

/// Trait for physical frame allocators.
pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame>;
    fn deallocate_frame(&mut self, frame: PhysFrame);
    fn free_frames(&self) -> usize;
    fn total_frames(&self) -> usize;
}
