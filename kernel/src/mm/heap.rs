use linked_list_allocator::LockedHeap;
use super::vmm;
use crate::arch::x86_64::boot;
use crate::serial_println;
use limine::memmap;

/// Kernel heap size: 8 MiB.
const HEAP_SIZE: u64 = 8 * 1024 * 1024;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Initialize the kernel heap by finding a contiguous usable memory region
/// large enough for the heap, above 1 MiB (avoiding BIOS/VGA holes).
pub fn init() {
    let hhdm = vmm::hhdm_offset();
    let mmap = boot::MEMORY_MAP.response().expect("No memory map");

    // Find a usable region >= HEAP_SIZE, starting above 1 MiB
    let mut heap_phys: Option<u64> = None;
    for entry in mmap.entries().iter() {
        if entry.type_ != memmap::MEMMAP_USABLE {
            continue;
        }
        // Skip regions below 1 MiB (BIOS, VGA holes)
        let region_start = if entry.base < 0x100000 {
            0x100000u64
        } else {
            entry.base
        };
        let region_end = entry.base + entry.length;
        if region_end <= region_start {
            continue;
        }
        let avail = region_end - region_start;
        if avail >= HEAP_SIZE {
            heap_phys = Some(region_start);
            break;
        }
    }

    let heap_start_phys = heap_phys.expect("No contiguous region large enough for heap");
    let heap_start = heap_start_phys + hhdm;

    serial_println!(
        "[heap] Initializing kernel heap at {:#x} (phys {:#x}), size: {} KiB",
        heap_start, heap_start_phys, HEAP_SIZE / 1024
    );

    unsafe {
        ALLOCATOR.lock().init(heap_start as *mut u8, HEAP_SIZE as usize);
    }
}
