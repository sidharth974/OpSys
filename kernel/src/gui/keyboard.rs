use core::sync::atomic::{AtomicUsize, AtomicU8, AtomicBool, Ordering};

/// Lock-free keyboard ring buffer.
/// The IRQ handler writes, the desktop thread reads. No mutex needed.
const BUF_SIZE: usize = 128;

static BUF: [AtomicU8; BUF_SIZE] = {
    const INIT: AtomicU8 = AtomicU8::new(0);
    [INIT; BUF_SIZE]
};
static HEAD: AtomicUsize = AtomicUsize::new(0); // IRQ writes here
static TAIL: AtomicUsize = AtomicUsize::new(0); // Desktop reads here
static SHIFT: AtomicBool = AtomicBool::new(false);
static CTRL: AtomicBool = AtomicBool::new(false);

/// Push a character into the buffer (called from IRQ context).
fn push(ch: u8) {
    let head = HEAD.load(Ordering::Relaxed);
    let next = (head + 1) % BUF_SIZE;
    if next != TAIL.load(Ordering::Relaxed) {
        BUF[head].store(ch, Ordering::Relaxed);
        HEAD.store(next, Ordering::Release);
    }
}

/// Pop a character from the buffer (called from desktop thread).
pub fn pop() -> Option<u8> {
    let tail = TAIL.load(Ordering::Relaxed);
    let head = HEAD.load(Ordering::Acquire);
    if tail == head {
        return None;
    }
    let ch = BUF[tail].load(Ordering::Relaxed);
    TAIL.store((tail + 1) % BUF_SIZE, Ordering::Release);
    Some(ch)
}

/// Called from the keyboard IRQ handler. No locks taken.
pub fn handle_interrupt(scancode: u8) {
    let pressed = scancode & 0x80 == 0;
    let code = scancode & 0x7F;

    match code {
        0x2A | 0x36 => { SHIFT.store(pressed, Ordering::Relaxed); return; }
        0x1D => { CTRL.store(pressed, Ordering::Relaxed); return; }
        0x38 => { return; } // Alt
        _ => {}
    }

    if !pressed { return; }

    let shift = SHIFT.load(Ordering::Relaxed);
    let ctrl = CTRL.load(Ordering::Relaxed);

    let ch = if shift {
        SCANCODE_SHIFT[code as usize]
    } else {
        SCANCODE_NORMAL[code as usize]
    };

    if ch == 0 { return; }

    if ctrl && (ch == b'c' || ch == b'C') {
        push(0x03); // Ctrl+C
        return;
    }

    push(ch);
}

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
    t[0x1C] = b'\n';
    t[0x1E] = b'a'; t[0x1F] = b's'; t[0x20] = b'd'; t[0x21] = b'f';
    t[0x22] = b'g'; t[0x23] = b'h'; t[0x24] = b'j'; t[0x25] = b'k';
    t[0x26] = b'l'; t[0x27] = b';'; t[0x28] = b'\'';
    t[0x29] = b'`';
    t[0x2B] = b'\\';
    t[0x2C] = b'z'; t[0x2D] = b'x'; t[0x2E] = b'c'; t[0x2F] = b'v';
    t[0x30] = b'b'; t[0x31] = b'n'; t[0x32] = b'm';
    t[0x33] = b','; t[0x34] = b'.'; t[0x35] = b'/';
    t[0x39] = b' ';
    t
};

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
