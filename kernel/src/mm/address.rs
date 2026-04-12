/// Physical address wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct PhysAddr(u64);

/// Virtual address wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct VirtAddr(u64);

impl PhysAddr {
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }

    pub const fn is_aligned(self, align: u64) -> bool {
        self.0 % align == 0
    }

    /// Convert to a virtual address using the HHDM offset.
    pub const fn to_virt(self, hhdm_offset: u64) -> VirtAddr {
        VirtAddr::new(self.0 + hhdm_offset)
    }
}

impl VirtAddr {
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }

    pub fn as_ptr<T>(self) -> *const T {
        self.0 as *const T
    }

    pub fn as_mut_ptr<T>(self) -> *mut T {
        self.0 as *mut T
    }
}
