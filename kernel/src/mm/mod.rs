pub mod address;
pub mod frame;
pub mod pmm;
pub mod vmm;
pub mod heap;

use crate::arch::x86_64::boot;
use crate::serial_println;

/// Initialize the entire memory management subsystem.
pub fn init() {
    let hhdm_response = boot::HHDM.response()
        .expect("HHDM response not available");
    let hhdm_offset = hhdm_response.offset;

    let mmap_response = boot::MEMORY_MAP.response()
        .expect("Memory map response not available");

    // Initialize the physical memory manager
    pmm::init(mmap_response, hhdm_offset);
    serial_println!("[mm] Physical memory manager initialized");

    // Initialize the kernel heap
    heap::init();
    serial_println!("[mm] Kernel heap initialized");
}
