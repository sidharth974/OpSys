use spin::Mutex;
use x86_64::instructions::port::Port;

/// Global mouse state.
pub static MOUSE: Mutex<MouseState> = Mutex::new(MouseState::new());

pub struct MouseState {
    pub x: i32,
    pub y: i32,
    pub buttons: u8,
    pub screen_w: i32,
    pub screen_h: i32,
    pub dirty: bool,
    cycle: u8,
    bytes: [u8; 3],
}

impl MouseState {
    const fn new() -> Self {
        Self {
            x: 0, y: 0, buttons: 0,
            screen_w: 1280, screen_h: 800,
            dirty: true,
            cycle: 0, bytes: [0; 3],
        }
    }

    pub fn set_screen_size(&mut self, w: i32, h: i32) {
        self.screen_w = w;
        self.screen_h = h;
        self.x = w / 2;
        self.y = h / 2;
    }

    pub fn process_byte(&mut self, byte: u8) -> bool {
        match self.cycle {
            0 => {
                if byte & 0x08 != 0 {
                    self.bytes[0] = byte;
                    self.cycle = 1;
                }
                false
            }
            1 => {
                self.bytes[1] = byte;
                self.cycle = 2;
                false
            }
            2 => {
                self.bytes[2] = byte;
                self.cycle = 0;
                self.decode_packet();
                true
            }
            _ => { self.cycle = 0; false }
        }
    }

    fn decode_packet(&mut self) {
        let flags = self.bytes[0];
        let old_buttons = self.buttons;
        let old_x = self.x;
        let old_y = self.y;

        self.buttons = flags & 0x07;

        let mut dx = self.bytes[1] as i32;
        if flags & 0x10 != 0 { dx -= 256; }

        let mut dy = self.bytes[2] as i32;
        if flags & 0x20 != 0 { dy -= 256; }
        dy = -dy;

        self.x = (self.x + dx).clamp(0, self.screen_w - 1);
        self.y = (self.y + dy).clamp(0, self.screen_h - 1);

        if self.x != old_x || self.y != old_y || self.buttons != old_buttons {
            self.dirty = true;
        }
    }
}

fn wait_input() {
    for _ in 0..10_000 {
        if unsafe { Port::<u8>::new(0x64).read() } & 2 == 0 { return; }
        core::hint::spin_loop();
    }
}

fn wait_output() {
    for _ in 0..10_000 {
        if unsafe { Port::<u8>::new(0x64).read() } & 1 != 0 { return; }
        core::hint::spin_loop();
    }
}

fn write_cmd(cmd: u8) {
    wait_input();
    unsafe { Port::<u8>::new(0x64).write(cmd); }
}

fn read_data() -> u8 {
    wait_output();
    unsafe { Port::<u8>::new(0x60).read() }
}

fn write_mouse(byte: u8) {
    write_cmd(0xD4);
    wait_input();
    unsafe { Port::<u8>::new(0x60).write(byte); }
    read_data(); // ACK
}

/// Initialize the PS/2 mouse.
pub fn init() {
    // Enable auxiliary (mouse) port
    write_cmd(0xA8);

    // Read controller config
    write_cmd(0x20);
    let mut config = read_data();
    config |= 0x02;  // Enable IRQ12
    config &= !0x20; // Enable mouse clock

    // Write controller config
    write_cmd(0x60);
    wait_input();
    unsafe { Port::<u8>::new(0x60).write(config); }

    // Set defaults
    write_mouse(0xF6);

    // Enable data reporting
    write_mouse(0xF4);
}

/// Called from IRQ12 handler.
pub fn handle_interrupt() {
    let byte = unsafe { Port::<u8>::new(0x60).read() };
    MOUSE.lock().process_byte(byte);
}
