use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use spin::Lazy;
use core::sync::atomic::{AtomicU64, Ordering};
use crate::arch::x86_64::gdt;
use crate::serial_println;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;
pub const TIMER_INTERRUPT_ID: u8 = PIC_1_OFFSET;
pub const KEYBOARD_INTERRUPT_ID: u8 = PIC_1_OFFSET + 1;
pub const MOUSE_INTERRUPT_ID: u8 = PIC_2_OFFSET + 4; // IRQ12

/// Tick counter: incremented every timer interrupt (~1ms with PIT default).
pub static TICKS: AtomicU64 = AtomicU64::new(0);

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    // CPU exceptions
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    }
    idt.page_fault.set_handler_fn(page_fault_handler);
    idt.general_protection_fault.set_handler_fn(general_protection_handler);

    // Hardware interrupts
    idt[TIMER_INTERRUPT_ID].set_handler_fn(timer_handler);
    idt[KEYBOARD_INTERRUPT_ID].set_handler_fn(keyboard_handler);
    idt[MOUSE_INTERRUPT_ID].set_handler_fn(mouse_handler);

    idt
});

pub fn init() {
    IDT.load();
}

// --- Exception Handlers ---

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    serial_println!("[EXCEPTION] BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[EXCEPTION] DOUBLE FAULT\n{:#?}", stack_frame);
    loop {
        x86_64::instructions::hlt();
    }
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let addr = x86_64::registers::control::Cr2::read();
    serial_println!(
        "[EXCEPTION] PAGE FAULT\nAccessed Address: {:?}\nError Code: {:?}\n{:#?}",
        addr,
        error_code,
        stack_frame
    );
    loop {
        x86_64::instructions::hlt();
    }
}

extern "x86-interrupt" fn general_protection_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    serial_println!(
        "[EXCEPTION] GENERAL PROTECTION FAULT\nError Code: {}\n{:#?}",
        error_code,
        stack_frame
    );
    loop {
        x86_64::instructions::hlt();
    }
}

// --- Hardware Interrupt Handlers ---

extern "x86-interrupt" fn timer_handler(_stack_frame: InterruptStackFrame) {
    let tick = TICKS.fetch_add(1, Ordering::Relaxed);

    // Run the scheduler every 10 ticks (~10ms quantum)
    if tick % 10 == 0 {
        SCHEDULER_READY.store(true, Ordering::Release);
    }

    unsafe {
        super::interrupts::PICS.lock().notify_end_of_interrupt(TIMER_INTERRUPT_ID);
    }
}

/// Flag to indicate the scheduler should run after the interrupt returns.
/// We can't call the scheduler directly from the interrupt handler because
/// we're holding the PIC lock. Instead, we set this flag and check it
/// in the main loop / yield points.
static SCHEDULER_READY: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(false);

/// Check if a reschedule is pending and perform it.
/// Called from yield points.
pub fn maybe_reschedule() {
    if SCHEDULER_READY.swap(false, Ordering::Acquire) {
        unsafe {
            crate::proc::scheduler::do_schedule();
        }
    }
}

extern "x86-interrupt" fn keyboard_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    let scancode: u8 = unsafe { Port::new(0x60).read() };
    crate::gui::keyboard::handle_interrupt(scancode);

    unsafe {
        super::interrupts::PICS.lock().notify_end_of_interrupt(KEYBOARD_INTERRUPT_ID);
    }
}

extern "x86-interrupt" fn mouse_handler(_stack_frame: InterruptStackFrame) {
    crate::gui::mouse::handle_interrupt();
    unsafe {
        super::interrupts::PICS.lock().notify_end_of_interrupt(MOUSE_INTERRUPT_ID);
    }
}
