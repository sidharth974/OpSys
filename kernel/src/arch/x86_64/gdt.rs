use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;
use spin::Lazy;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
const STACK_SIZE: usize = 4096 * 4;

static mut DOUBLE_FAULT_STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

/// Per-CPU kernel stack for syscall entry (RSP0 in TSS).
pub static mut SYSCALL_KERNEL_STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
pub const SYSCALL_STACK_SIZE: usize = STACK_SIZE;

static TSS: Lazy<TaskStateSegment> = Lazy::new(|| {
    let mut tss = TaskStateSegment::new();
    // IST[0] = double fault stack
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
        let stack_start = VirtAddr::from_ptr(&raw const DOUBLE_FAULT_STACK as *const u8);
        stack_start + STACK_SIZE as u64
    };
    // RSP0 = kernel stack for ring-3 -> ring-0 transitions
    tss.privilege_stack_table[0] = {
        let stack_start = VirtAddr::from_ptr(&raw const SYSCALL_KERNEL_STACK as *const u8);
        stack_start + STACK_SIZE as u64
    };
    tss
});

static GDT: Lazy<(GlobalDescriptorTable, Selectors)> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();
    // Index 1: Kernel code (ring 0) - MUST be at index 1 for SYSCALL
    let kernel_code = gdt.append(Descriptor::kernel_code_segment());
    // Index 2: Kernel data (ring 0)
    let kernel_data = gdt.append(Descriptor::kernel_data_segment());
    // Index 3: User data (ring 3) - MUST be right before user code for SYSRET
    let user_data = gdt.append(Descriptor::user_data_segment());
    // Index 4: User code (ring 3) - MUST be right after user data for SYSRET
    let user_code = gdt.append(Descriptor::user_code_segment());
    // Index 5-6: TSS (takes 2 entries)
    let tss_selector = gdt.append(Descriptor::tss_segment(&TSS));
    (gdt, Selectors {
        kernel_code,
        kernel_data,
        user_code,
        user_data,
        tss: tss_selector,
    })
});

pub struct Selectors {
    pub kernel_code: SegmentSelector,
    pub kernel_data: SegmentSelector,
    pub user_code: SegmentSelector,
    pub user_data: SegmentSelector,
    pub tss: SegmentSelector,
}

pub fn selectors() -> &'static Selectors {
    &GDT.1
}

pub fn init() {
    use x86_64::instructions::segmentation::{CS, DS, SS, Segment};
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.kernel_code);
        DS::set_reg(SegmentSelector(0));
        SS::set_reg(SegmentSelector(0));
        load_tss(GDT.1.tss);
    }
}
