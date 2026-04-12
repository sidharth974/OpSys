pub mod boot;
pub mod serial;
pub mod gdt;
pub mod idt;
pub mod interrupts;
pub mod cpu;

use crate::serial_println;

/// Initialize all architecture-specific subsystems.
pub fn init() {
    serial::SERIAL1.lock().init();
    serial_println!("[arch] Serial console initialized");

    gdt::init();
    serial_println!("[arch] GDT initialized");

    idt::init();
    serial_println!("[arch] IDT initialized");

    interrupts::init_pics();
    serial_println!("[arch] PIC initialized");

    cpu::detect_features();

    x86_64::instructions::interrupts::enable();
    serial_println!("[arch] Interrupts enabled");
}
