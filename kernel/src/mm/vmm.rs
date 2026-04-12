use super::address::{PhysAddr, VirtAddr};
use crate::arch::x86_64::boot;

/// Get the HHDM offset used for physical-to-virtual translation.
pub fn hhdm_offset() -> u64 {
    boot::HHDM.response()
        .expect("HHDM not available")
        .offset
}

/// Convert a physical address to its HHDM virtual address.
pub fn phys_to_virt(phys: PhysAddr) -> VirtAddr {
    phys.to_virt(hhdm_offset())
}
