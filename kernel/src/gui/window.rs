use alloc::string::String;
use alloc::vec::Vec;

/// A styled line of text in a window.
pub struct StyledLine {
    pub text: String,
    pub color: Option<u32>,
}

/// A window in the window manager.
pub struct Window {
    pub id: usize,
    pub title: String,
    pub kind: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub visible: bool,
    pub active: bool,
    pub lines: Vec<StyledLine>,
    /// For terminal windows: is this an interactive terminal?
    pub is_terminal: bool,
    /// Current input line for terminal windows.
    pub input_buf: String,
}

impl Window {
    pub fn new(id: usize, title: &str, kind: &str, x: i32, y: i32, w: i32, h: i32) -> Self {
        Self {
            id,
            title: String::from(title),
            kind: String::from(kind),
            x, y, width: w, height: h,
            visible: true,
            active: false,
            lines: Vec::new(),
            is_terminal: false,
            input_buf: String::new(),
        }
    }

    pub fn add_line(&mut self, text: &str) {
        self.lines.push(StyledLine { text: String::from(text), color: None });
        let max = ((self.height - 40) / 16) as usize;
        while self.lines.len() > max { self.lines.remove(0); }
    }

    pub fn add_styled(&mut self, text: &str, color: u32) {
        self.lines.push(StyledLine { text: String::from(text), color: Some(color) });
        let max = ((self.height - 40) / 16) as usize;
        while self.lines.len() > max { self.lines.remove(0); }
    }

    pub fn title_bar_contains(&self, px: i32, py: i32) -> bool {
        px >= self.x + 60 && px < self.x + self.width &&
        py >= self.y && py < self.y + 28
    }

    pub fn close_contains(&self, px: i32, py: i32) -> bool {
        // macOS-style: close button is left side at (x+10, y+8), 12x12
        px >= self.x + 8 && px < self.x + 24 &&
        py >= self.y + 6 && py < self.y + 22
    }

    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && px < self.x + self.width &&
        py >= self.y && py < self.y + self.height
    }
}
