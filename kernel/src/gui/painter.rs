use alloc::vec::Vec;
use crate::drivers::framebuffer::font;

/// Double-buffered framebuffer painter.
/// All drawing goes to a back buffer. Call `flip()` to copy to the real framebuffer.
pub struct Painter {
    front: *mut u8,  // Real framebuffer
    back: Vec<u32>,  // Back buffer (owned)
    width: u32,
    height: u32,
    pitch: u32,      // Front buffer pitch in bytes
}

unsafe impl Send for Painter {}

impl Painter {
    pub fn new(fb: *mut u8, width: u32, height: u32, pitch: u32) -> Self {
        let pixels = (width * height) as usize;
        let back = alloc::vec![0u32; pixels];
        Self { front: fb, back, width, height, pitch }
    }

    /// Copy back buffer to front buffer (the real framebuffer). No flicker.
    pub fn flip(&self) {
        for y in 0..self.height {
            let src_offset = (y * self.width) as usize;
            let dst_offset = (y * self.pitch) as usize;
            let src = &self.back[src_offset..src_offset + self.width as usize];
            let dst = unsafe { self.front.add(dst_offset) as *mut u32 };
            unsafe {
                core::ptr::copy_nonoverlapping(src.as_ptr(), dst, self.width as usize);
            }
        }
    }

    #[inline(always)]
    pub fn put_pixel(&mut self, x: i32, y: i32, color: u32) {
        if x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height {
            self.back[(y as u32 * self.width + x as u32) as usize] = color;
        }
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u32) {
        let x0 = x.max(0) as u32;
        let y0 = y.max(0) as u32;
        let x1 = ((x + w) as u32).min(self.width);
        let y1 = ((y + h) as u32).min(self.height);
        for row in y0..y1 {
            let off = (row * self.width) as usize;
            for col in x0..x1 {
                self.back[off + col as usize] = color;
            }
        }
    }

    pub fn draw_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u32) {
        for dx in 0..w { self.put_pixel(x + dx, y, color); self.put_pixel(x + dx, y + h - 1, color); }
        for dy in 0..h { self.put_pixel(x, y + dy, color); self.put_pixel(x + w - 1, y + dy, color); }
    }

    pub fn draw_char(&mut self, x: i32, y: i32, ch: u8, fg: u32, bg: u32) {
        let glyph = font::get_glyph(ch);
        for (dy, &row) in glyph.iter().enumerate() {
            for dx in 0..8 {
                let color = if row & (0x80 >> dx) != 0 { fg } else { bg };
                self.put_pixel(x + dx, y + dy as i32, color);
            }
        }
    }

    pub fn draw_text(&mut self, x: i32, y: i32, text: &str, fg: u32, bg: u32) {
        for (i, byte) in text.bytes().enumerate() {
            self.draw_char(x + (i as i32) * 8, y, byte, fg, bg);
        }
    }

    pub fn draw_text_transparent(&mut self, x: i32, y: i32, text: &str, fg: u32) {
        for (i, byte) in text.bytes().enumerate() {
            let glyph = font::get_glyph(byte);
            for (dy, &row) in glyph.iter().enumerate() {
                for dx in 0..8 {
                    if row & (0x80 >> dx) != 0 {
                        self.put_pixel(x + (i as i32) * 8 + dx, y + dy as i32, fg);
                    }
                }
            }
        }
    }

    /// Draw a rounded-corner rectangle (approximated with filled corners).
    pub fn fill_rounded_rect(&mut self, x: i32, y: i32, w: i32, h: i32, r: i32, color: u32) {
        // Main body
        self.fill_rect(x + r, y, w - 2 * r, h, color);
        self.fill_rect(x, y + r, r, h - 2 * r, color);
        self.fill_rect(x + w - r, y + r, r, h - 2 * r, color);
        // Corners (simple quarter-circle fill)
        for dy in 0..r {
            for dx in 0..r {
                if dx * dx + dy * dy <= r * r {
                    self.put_pixel(x + r - dx, y + r - dy, color);       // top-left
                    self.put_pixel(x + w - r - 1 + dx, y + r - dy, color); // top-right
                    self.put_pixel(x + r - dx, y + h - r - 1 + dy, color); // bottom-left
                    self.put_pixel(x + w - r - 1 + dx, y + h - r - 1 + dy, color); // bottom-right
                }
            }
        }
    }

    pub fn draw_cursor(&mut self, x: i32, y: i32) {
        const C: [u16; 16] = [
            0b1000000000000000, 0b1100000000000000,
            0b1110000000000000, 0b1111000000000000,
            0b1111100000000000, 0b1111110000000000,
            0b1111111000000000, 0b1111111100000000,
            0b1111111110000000, 0b1111110000000000,
            0b1101110000000000, 0b1000111000000000,
            0b0000111000000000, 0b0000011100000000,
            0b0000011100000000, 0b0000000000000000,
        ];
        // Black outline
        for (dy, &row) in C.iter().enumerate() {
            for dx in 0..12 {
                if row & (0x8000 >> dx) != 0 {
                    self.put_pixel(x + dx + 1, y + dy as i32 + 1, 0x00000000);
                }
            }
        }
        // White fill
        for (dy, &row) in C.iter().enumerate() {
            for dx in 0..12 {
                if row & (0x8000 >> dx) != 0 {
                    self.put_pixel(x + dx, y + dy as i32, 0x00FFFFFF);
                }
            }
        }
    }

    /// Save the pixels under a 16x16 area from the FRONT buffer.
    pub fn save_front_area(&self, x: i32, y: i32, buf: &mut [u32; 16 * 16]) {
        for dy in 0..16i32 {
            for dx in 0..16i32 {
                let px = x + dx;
                let py = y + dy;
                if px >= 0 && py >= 0 && (px as u32) < self.width && (py as u32) < self.height {
                    let off = (py as u32 * self.pitch / 4 + px as u32) as usize;
                    buf[(dy * 16 + dx) as usize] = unsafe {
                        (self.front as *const u32).add(off).read_volatile()
                    };
                }
            }
        }
    }

    /// Restore a 16x16 area to the FRONT buffer.
    pub fn restore_front_area(&self, x: i32, y: i32, buf: &[u32; 16 * 16]) {
        for dy in 0..16i32 {
            for dx in 0..16i32 {
                let px = x + dx;
                let py = y + dy;
                if px >= 0 && py >= 0 && (px as u32) < self.width && (py as u32) < self.height {
                    let off = (py as u32 * self.pitch / 4 + px as u32) as usize;
                    unsafe {
                        (self.front as *mut u32).add(off).write_volatile(buf[(dy * 16 + dx) as usize]);
                    }
                }
            }
        }
    }

    /// Draw cursor directly to the FRONT buffer (fast path).
    pub fn draw_cursor_front(&self, x: i32, y: i32) {
        const C: [u16; 16] = [
            0b1000000000000000, 0b1100000000000000,
            0b1110000000000000, 0b1111000000000000,
            0b1111100000000000, 0b1111110000000000,
            0b1111111000000000, 0b1111111100000000,
            0b1111111110000000, 0b1111110000000000,
            0b1101110000000000, 0b1000111000000000,
            0b0000111000000000, 0b0000011100000000,
            0b0000011100000000, 0b0000000000000000,
        ];
        for (dy, &row) in C.iter().enumerate() {
            for dx in 0..12 {
                if row & (0x8000 >> dx) != 0 {
                    let px = x + dx;
                    let py = y + dy as i32;
                    if px >= 0 && py >= 0 && (px as u32) < self.width && (py as u32) < self.height {
                        let off = (py as u32 * self.pitch / 4 + px as u32) as usize;
                        unsafe { (self.front as *mut u32).add(off).write_volatile(0x00FFFFFF); }
                    }
                }
            }
        }
    }

    pub fn screen_width(&self) -> i32 { self.width as i32 }
    pub fn screen_height(&self) -> i32 { self.height as i32 }
}
