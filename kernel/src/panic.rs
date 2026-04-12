use core::panic::PanicInfo;
use crate::serial_println;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!();
    serial_println!("========== KERNEL PANIC ==========");
    serial_println!("{}", info);
    serial_println!("==================================");

    loop {
        x86_64::instructions::hlt();
    }
}
