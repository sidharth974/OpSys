use spin::Mutex;

/// Global keyboard state.
pub static KEYBOARD: Mutex<KeyboardState> = Mutex::new(KeyboardState::new());

pub struct KeyboardState {
    /// Ring buffer of key events.
    pub buffer: [u8; 64],
    pub head: usize,
    pub tail: usize,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

impl KeyboardState {
    const fn new() -> Self {
        Self {
            buffer: [0; 64],
            head: 0, tail: 0,
            shift: false, ctrl: false, alt: false,
        }
    }

    /// Push an ASCII character into the buffer.
    pub fn push(&mut self, ch: u8) {
        let next = (self.head + 1) % self.buffer.len();
        if next != self.tail {
            self.buffer[self.head] = ch;
            self.head = next;
        }
    }

    /// Pop a character from the buffer. Returns None if empty.
    pub fn pop(&mut self) -> Option<u8> {
        if self.head == self.tail {
            return None;
        }
        let ch = self.buffer[self.tail];
        self.tail = (self.tail + 1) % self.buffer.len();
        Some(ch)
    }

    /// Process a PS/2 scancode (set 1) and push the ASCII equivalent.
    pub fn process_scancode(&mut self, scancode: u8) {
        let pressed = scancode & 0x80 == 0;
        let code = scancode & 0x7F;

        match code {
            0x2A | 0x36 => { self.shift = pressed; return; }  // Shift
            0x1D => { self.ctrl = pressed; return; }           // Ctrl
            0x38 => { self.alt = pressed; return; }            // Alt
            _ => {}
        }

        if !pressed { return; }

        let ch = if self.shift {
            SCANCODE_SHIFT[code as usize]
        } else {
            SCANCODE_NORMAL[code as usize]
        };

        if ch == 0 { return; }

        // Ctrl+C
        if self.ctrl && ch == b'c' {
            self.push(0x03);
            return;
        }

        self.push(ch);
    }
}

/// Called from the keyboard IRQ handler.
pub fn handle_interrupt(scancode: u8) {
    KEYBOARD.lock().process_scancode(scancode);
}

/// PS/2 scancode set 1 -> ASCII (normal).
static SCANCODE_NORMAL: [u8; 128] = {
    let mut t = [0u8; 128];
    t[0x01] = 0x1B; // Escape
    t[0x02] = b'1'; t[0x03] = b'2'; t[0x04] = b'3'; t[0x05] = b'4';
    t[0x06] = b'5'; t[0x07] = b'6'; t[0x08] = b'7'; t[0x09] = b'8';
    t[0x0A] = b'9'; t[0x0B] = b'0'; t[0x0C] = b'-'; t[0x0D] = b'=';
    t[0x0E] = 0x08; // Backspace
    t[0x0F] = b'\t';
    t[0x10] = b'q'; t[0x11] = b'w'; t[0x12] = b'e'; t[0x13] = b'r';
    t[0x14] = b't'; t[0x15] = b'y'; t[0x16] = b'u'; t[0x17] = b'i';
    t[0x18] = b'o'; t[0x19] = b'p'; t[0x1A] = b'['; t[0x1B] = b']';
    t[0x1C] = b'\n'; // Enter
    t[0x1E] = b'a'; t[0x1F] = b's'; t[0x20] = b'd'; t[0x21] = b'f';
    t[0x22] = b'g'; t[0x23] = b'h'; t[0x24] = b'j'; t[0x25] = b'k';
    t[0x26] = b'l'; t[0x27] = b';'; t[0x28] = b'\'';
    t[0x29] = b'`';
    t[0x2B] = b'\\';
    t[0x2C] = b'z'; t[0x2D] = b'x'; t[0x2E] = b'c'; t[0x2F] = b'v';
    t[0x30] = b'b'; t[0x31] = b'n'; t[0x32] = b'm';
    t[0x33] = b','; t[0x34] = b'.'; t[0x35] = b'/';
    t[0x39] = b' '; // Space
    t
};

/// PS/2 scancode set 1 -> ASCII (shift held).
static SCANCODE_SHIFT: [u8; 128] = {
    let mut t = [0u8; 128];
    t[0x01] = 0x1B;
    t[0x02] = b'!'; t[0x03] = b'@'; t[0x04] = b'#'; t[0x05] = b'$';
    t[0x06] = b'%'; t[0x07] = b'^'; t[0x08] = b'&'; t[0x09] = b'*';
    t[0x0A] = b'('; t[0x0B] = b')'; t[0x0C] = b'_'; t[0x0D] = b'+';
    t[0x0E] = 0x08;
    t[0x0F] = b'\t';
    t[0x10] = b'Q'; t[0x11] = b'W'; t[0x12] = b'E'; t[0x13] = b'R';
    t[0x14] = b'T'; t[0x15] = b'Y'; t[0x16] = b'U'; t[0x17] = b'I';
    t[0x18] = b'O'; t[0x19] = b'P'; t[0x1A] = b'{'; t[0x1B] = b'}';
    t[0x1C] = b'\n';
    t[0x1E] = b'A'; t[0x1F] = b'S'; t[0x20] = b'D'; t[0x21] = b'F';
    t[0x22] = b'G'; t[0x23] = b'H'; t[0x24] = b'J'; t[0x25] = b'K';
    t[0x26] = b'L'; t[0x27] = b':'; t[0x28] = b'"';
    t[0x29] = b'~';
    t[0x2B] = b'|';
    t[0x2C] = b'Z'; t[0x2D] = b'X'; t[0x2E] = b'C'; t[0x2F] = b'V';
    t[0x30] = b'B'; t[0x31] = b'N'; t[0x32] = b'M';
    t[0x33] = b'<'; t[0x34] = b'>'; t[0x35] = b'?';
    t[0x39] = b' ';
    t
};
