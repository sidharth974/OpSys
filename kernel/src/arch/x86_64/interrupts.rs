use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::instructions::port::Port;
use super::idt::PIC_1_OFFSET;

const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe {
    ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
});

pub fn init_pics() {
    unsafe {
        PICS.lock().initialize();
    }

    // Unmask IRQ2 (cascade to slave PIC) and IRQ12 (PS/2 mouse)
    // PIC1 mask: bit 2 = IRQ2 cascade, must be 0 (unmasked)
    // PIC2 mask: bit 4 = IRQ12 (mouse), must be 0 (unmasked)
    unsafe {
        let mask1 = Port::<u8>::new(0x21).read();
        Port::<u8>::new(0x21).write(mask1 & !0x04); // Unmask IRQ2 (cascade)

        let mask2 = Port::<u8>::new(0xA1).read();
        Port::<u8>::new(0xA1).write(mask2 & !0x10); // Unmask IRQ12 (mouse)
    }
}
