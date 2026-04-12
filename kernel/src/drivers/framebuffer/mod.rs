use spin::Mutex;
use crate::arch::x86_64::boot;
use crate::serial_println;

pub static CONSOLE: Mutex<Option<FbConsole>> = Mutex::new(None);

/// Built-in 8x16 bitmap font (ASCII 32-126).
/// Each character is 16 bytes (16 rows of 8 pixels).
pub mod font;

/// Framebuffer console: text rendering on a pixel framebuffer.
pub struct FbConsole {
    pub fb_base: *mut u8,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub bpp: u16,
    /// Text cursor position (in character cells).
    col: u32,
    row: u32,
    /// Max characters per row/column.
    cols: u32,
    rows: u32,
    /// Text color (RGB).
    fg: u32,
    bg: u32,
}

unsafe impl Send for FbConsole {}

impl FbConsole {
    /// Initialize from the Limine framebuffer response.
    pub fn init() -> Option<Self> {
        let fb_response = boot::FRAMEBUFFER.response()?;
        let framebuffers = fb_response.framebuffers();
        let fb = framebuffers.first()?;

        let base = fb.address() as *mut u8;
        let width = fb.width as u32;
        let height = fb.height as u32;
        let pitch = fb.pitch as u32;
        let bpp = fb.bpp;

        let cols = width / 8;
        let rows = height / 16;

        serial_println!("[fb] Framebuffer: {}x{}, {} bpp, pitch {}", width, height, bpp, pitch);
        serial_println!("[fb] Text console: {}x{} characters", cols, rows);

        let mut console = Self {
            fb_base: base,
            width,
            height,
            pitch,
            bpp,
            col: 0,
            row: 0,
            cols,
            rows,
            fg: 0x00CC88FF, // Light purple
            bg: 0x00101020, // Dark blue-black
        };

        // Clear screen
        console.clear();

        // Draw a welcome banner
        console.set_color(0x0000CCFF, 0x00101020); // Cyan
        console.write_str("OpSys v0.1.0 - AI-Optimized Microkernel\n");
        console.set_color(0x00888888, 0x00101020); // Grey
        console.write_str("Framebuffer console active\n\n");
        console.set_color(0x00CC88FF, 0x00101020); // Default

        Some(console)
    }

    /// Clear the entire screen with the background color.
    pub fn clear(&mut self) {
        // Fast bulk clear using 32-bit writes per row
        let pixels_per_row = self.width;
        for y in 0..self.height {
            let row_offset = (y * self.pitch) as usize;
            let row_ptr = unsafe { self.fb_base.add(row_offset) as *mut u32 };
            for x in 0..pixels_per_row {
                unsafe { row_ptr.add(x as usize).write_volatile(self.bg); }
            }
        }
        self.col = 0;
        self.row = 0;
    }

    /// Put a single pixel at (x, y).
    fn put_pixel(&self, x: u32, y: u32, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }
        let offset = (y * self.pitch + x * (self.bpp as u32 / 8)) as usize;
        unsafe {
            let pixel = self.fb_base.add(offset) as *mut u32;
            pixel.write_volatile(color);
        }
    }

    /// Draw a single character at the given cell position.
    fn draw_char(&self, col: u32, row: u32, ch: u8) {
        let glyph = font::get_glyph(ch);
        let x0 = col * 8;
        let y0 = row * 16;

        for (dy, &glyph_row) in glyph.iter().enumerate() {
            for dx in 0..8u32 {
                let color = if glyph_row & (0x80 >> dx) != 0 {
                    self.fg
                } else {
                    self.bg
                };
                self.put_pixel(x0 + dx, y0 + dy as u32, color);
            }
        }
    }

    /// Scroll the screen up by one row.
    fn scroll(&mut self) {
        let row_bytes = self.pitch * 16;
        unsafe {
            // Copy rows 1..n to 0..n-1
            core::ptr::copy(
                self.fb_base.add(row_bytes as usize),
                self.fb_base,
                ((self.rows - 1) * row_bytes) as usize,
            );
            // Clear the last row
            let last_row_start = self.fb_base.add(((self.rows - 1) * row_bytes) as usize);
            core::ptr::write_bytes(last_row_start, 0x10, row_bytes as usize);
        }
        self.row = self.rows - 1;
    }

    /// Set text colors.
    pub fn set_color(&mut self, fg: u32, bg: u32) {
        self.fg = fg;
        self.bg = bg;
    }

    /// Write a byte to the console.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.col = 0;
                self.row += 1;
                if self.row >= self.rows {
                    self.scroll();
                }
            }
            b'\r' => {
                self.col = 0;
            }
            0x08 => {
                // Backspace
                if self.col > 0 {
                    self.col -= 1;
                    self.draw_char(self.col, self.row, b' ');
                }
            }
            byte => {
                if self.col >= self.cols {
                    self.col = 0;
                    self.row += 1;
                    if self.row >= self.rows {
                        self.scroll();
                    }
                }
                self.draw_char(self.col, self.row, byte);
                self.col += 1;
            }
        }
    }

    /// Write a string to the console.
    pub fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }
}

/// Initialize the framebuffer console.
pub fn init() {
    if let Some(console) = FbConsole::init() {
        *CONSOLE.lock() = Some(console);
    }
}

/// Write a string to the framebuffer console (if available).
pub fn write(s: &str) {
    if let Some(ref mut console) = *CONSOLE.lock() {
        console.write_str(s);
    }
}
