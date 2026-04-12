use core::fmt;
use spin::Mutex;
use x86_64::instructions::port::Port;

const COM1: u16 = 0x3F8;

pub static SERIAL1: Mutex<SerialPort> = Mutex::new(SerialPort::new(COM1));

pub struct SerialPort {
    data: Port<u8>,
    int_enable: Port<u8>,
    fifo_ctrl: Port<u8>,
    line_ctrl: Port<u8>,
    modem_ctrl: Port<u8>,
    line_status: Port<u8>,
}

impl SerialPort {
    const fn new(base: u16) -> Self {
        Self {
            data: Port::new(base),
            int_enable: Port::new(base + 1),
            fifo_ctrl: Port::new(base + 2),
            line_ctrl: Port::new(base + 3),
            modem_ctrl: Port::new(base + 4),
            line_status: Port::new(base + 5),
        }
    }

    /// Initialize the serial port with 115200 baud, 8N1.
    pub fn init(&mut self) {
        unsafe {
            // Disable interrupts
            self.int_enable.write(0x00);
            // Enable DLAB (set baud rate divisor)
            self.line_ctrl.write(0x80);
            // Divisor = 1 (115200 baud) - low byte
            self.data.write(0x01);
            // Divisor high byte
            self.int_enable.write(0x00);
            // 8 bits, no parity, one stop bit (8N1)
            self.line_ctrl.write(0x03);
            // Enable FIFO, clear them, 14-byte threshold
            self.fifo_ctrl.write(0xC7);
            // IRQs enabled, RTS/DSR set
            self.modem_ctrl.write(0x0B);
        }
    }

    fn is_transmit_empty(&mut self) -> bool {
        unsafe { self.line_status.read() & 0x20 != 0 }
    }

    pub fn send(&mut self, byte: u8) {
        while !self.is_transmit_empty() {
            core::hint::spin_loop();
        }
        unsafe {
            self.data.write(byte);
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            if byte == b'\n' {
                self.send(b'\r');
            }
            self.send(byte);
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::arch::x86_64::serial::_serial_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _serial_print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        SERIAL1.lock().write_fmt(args).expect("serial print failed");
    });
}
