use super::painter::Painter;

/// OpSys "Ember" theme — inspired by modern dark IDEs.
/// Deep blacks, warm amber/orange accents, clean spacing.
pub mod colors {
    pub const DESKTOP_BG: u32     = 0x00111111; // Near black
    pub const TASKBAR_BG: u32     = 0x001A1A1A; // Slightly lighter
    pub const WINDOW_BG: u32      = 0x001E1E1E; // Window body
    pub const TITLE_BAR: u32      = 0x002A2A2A; // Inactive title
    pub const TITLE_ACTIVE: u32   = 0x00333333; // Active title
    pub const TEXT_WHITE: u32     = 0x00E5E5E5; // Primary text
    pub const TEXT_DIM: u32       = 0x00888888; // Secondary text
    pub const TEXT_CYAN: u32      = 0x0066D9EF; // Cyan highlights
    pub const TEXT_GREEN: u32     = 0x00A6E22E; // Success green
    pub const TEXT_YELLOW: u32    = 0x00E6DB74; // Warm yellow
    pub const TEXT_RED: u32       = 0x00F92672; // Error/close red
    pub const TEXT_PURPLE: u32    = 0x00AE81FF; // Purple accent
    pub const TEXT_ORANGE: u32    = 0x00FD971F; // Orange (primary accent)
    pub const ACCENT: u32         = 0x00D4A056; // Warm amber accent
    pub const BORDER_DIM: u32     = 0x00333333; // Subtle border
    pub const BORDER_BRIGHT: u32  = 0x00555555; // Focused border
    pub const BTN_BG: u32         = 0x00282828;
    pub const BTN_HOVER: u32      = 0x003E3E3E;
    pub const CLOSE_BTN: u32      = 0x00E84040;
    pub const MAX_BTN: u32        = 0x0050C050;
    pub const MIN_BTN: u32        = 0x00E8B838;
    pub const MENU_BG: u32        = 0x00252525;
    pub const MENU_HOVER: u32     = 0x00D4A056; // Amber highlight
    pub const MENU_HOVER_TEXT: u32= 0x00111111; // Dark text on amber
    pub const ICON_BG: u32        = 0x001E1E1E;
    pub const SELECTION: u32      = 0x00D4A056;
    pub const SCROLLBAR: u32      = 0x00404040;
    pub const SEPARATOR: u32      = 0x002A2A2A;
}

/// Draw a modern window with thin title bar.
pub fn draw_window(p: &mut Painter, x: i32, y: i32, w: i32, h: i32, title: &str, active: bool) {
    let title_bg = if active { colors::TITLE_ACTIVE } else { colors::TITLE_BAR };
    let border = if active { colors::BORDER_BRIGHT } else { colors::BORDER_DIM };

    // Shadow
    p.fill_rect(x + 2, y + 2, w, h, 0x00080808);

    // Window body
    p.fill_rect(x, y, w, h, colors::WINDOW_BG);

    // Title bar (28px)
    p.fill_rect(x, y, w, 28, title_bg);

    // Traffic light buttons (left side, macOS style)
    let by = y + 8;
    // Close
    p.fill_rounded_rect(x + 10, by, 12, 12, 6, colors::CLOSE_BTN);
    // Minimize
    p.fill_rounded_rect(x + 28, by, 12, 12, 6, colors::MIN_BTN);
    // Maximize
    p.fill_rounded_rect(x + 46, by, 12, 12, 6, colors::MAX_BTN);

    // Title text (centered)
    let title_display: alloc::string::String = if title.len() > 40 {
        alloc::format!("{}...", &title[..37])
    } else {
        alloc::string::String::from(title)
    };
    let tw = title_display.len() as i32 * 8;
    let tx = x + (w - tw) / 2;
    p.draw_text(tx, y + 7, &title_display, if active { colors::TEXT_WHITE } else { colors::TEXT_DIM }, title_bg);

    // Border (1px)
    p.draw_rect(x, y, w, h, border);
    // Title separator
    p.fill_rect(x + 1, y + 28, w - 2, 1, border);
}

/// Draw a button.
pub fn draw_button(p: &mut Painter, x: i32, y: i32, w: i32, h: i32, label: &str, hovered: bool) {
    let bg = if hovered { colors::BTN_HOVER } else { colors::BTN_BG };
    p.fill_rounded_rect(x, y, w, h, 4, bg);
    if hovered {
        p.draw_rect(x, y, w, h, colors::ACCENT);
    } else {
        p.draw_rect(x, y, w, h, colors::BORDER_DIM);
    }
    let tx = x + (w - label.len() as i32 * 8) / 2;
    p.draw_text(tx, y + (h - 16) / 2, label, colors::TEXT_WHITE, bg);
}

/// Draw a progress bar.
pub fn draw_progress(p: &mut Painter, x: i32, y: i32, w: i32, frac: f32, color: u32, label: &str) {
    p.fill_rect(x, y, w, 12, 0x00181818);
    let filled = (w as f32 * frac.clamp(0.0, 1.0)) as i32;
    if filled > 0 { p.fill_rect(x, y, filled, 12, color); }
    p.draw_rect(x, y, w, 12, colors::BORDER_DIM);
    p.draw_text_transparent(x + 4, y - 1, label, colors::TEXT_WHITE);
}

/// Draw the taskbar.
pub fn draw_taskbar(p: &mut Painter, y: i32, w: i32, start_hovered: bool) {
    let h = 36;
    // Taskbar bg
    p.fill_rect(0, y, w, h, colors::TASKBAR_BG);
    // Top border
    p.fill_rect(0, y, w, 1, colors::BORDER_DIM);

    // Start button — rounded, amber accent on hover
    let sbg = if start_hovered { colors::ACCENT } else { colors::BTN_BG };
    let stxt = if start_hovered { colors::MENU_HOVER_TEXT } else { colors::TEXT_ORANGE };
    p.fill_rounded_rect(6, y + 5, 78, 26, 4, sbg);
    if !start_hovered { p.draw_rect(6, y + 5, 78, 26, colors::BORDER_DIM); }
    // Diamond icon
    p.put_pixel(16, y + 14, colors::TEXT_ORANGE);
    p.put_pixel(15, y + 15, colors::TEXT_ORANGE); p.put_pixel(16, y + 15, colors::TEXT_ORANGE); p.put_pixel(17, y + 15, colors::TEXT_ORANGE);
    p.put_pixel(14, y + 16, colors::TEXT_ORANGE); p.put_pixel(15, y + 16, colors::TEXT_ORANGE);
    p.put_pixel(16, y + 16, colors::TEXT_ORANGE); p.put_pixel(17, y + 16, colors::TEXT_ORANGE); p.put_pixel(18, y + 16, colors::TEXT_ORANGE);
    p.put_pixel(15, y + 17, colors::TEXT_ORANGE); p.put_pixel(16, y + 17, colors::TEXT_ORANGE); p.put_pixel(17, y + 17, colors::TEXT_ORANGE);
    p.put_pixel(16, y + 18, colors::TEXT_ORANGE);
    p.draw_text(24, y + 10, "OpSys", stxt, sbg);

    // Separator
    p.fill_rect(90, y + 6, 1, 24, colors::SEPARATOR);
}

/// Draw the start menu.
pub fn draw_start_menu(p: &mut Painter, x: i32, y: i32, items: &[(&str, &str)], hover_idx: Option<usize>) {
    let w = 240;
    let item_h = 34;
    let pad = 6;
    let h = items.len() as i32 * item_h + 2 * pad;

    // Shadow
    p.fill_rect(x + 3, y - h + 3, w, h, 0x00050505);
    // Body
    p.fill_rounded_rect(x, y - h, w, h, 6, colors::MENU_BG);
    p.draw_rect(x, y - h, w, h, colors::BORDER_BRIGHT);

    for (i, (icon_key, label)) in items.iter().enumerate() {
        let iy = y - h + pad + i as i32 * item_h;
        let hovered = hover_idx == Some(i);

        if hovered {
            p.fill_rounded_rect(x + 4, iy, w - 8, item_h - 2, 4, colors::MENU_HOVER);
        }

        let icon_color = match *icon_key {
            "sys" => colors::TEXT_CYAN,
            "ai" => colors::TEXT_PURPLE,
            "hw" => colors::TEXT_GREEN,
            "set" => colors::TEXT_ORANGE,
            "term" => colors::TEXT_WHITE,
            "pwr" => colors::TEXT_RED,
            _ => colors::TEXT_DIM,
        };

        let bg = if hovered { colors::MENU_HOVER } else { colors::MENU_BG };
        let fg = if hovered { colors::MENU_HOVER_TEXT } else { colors::TEXT_WHITE };

        // Icon circle
        p.fill_rounded_rect(x + 12, iy + 5, 24, 24, 12, icon_color);
        p.draw_text(x + 20, iy + 9, &icon_key[..1].to_ascii_uppercase(), 0x00111111, icon_color);

        // Label
        p.draw_text(x + 46, iy + 9, label, fg, bg);
    }
}

/// Draw a desktop icon.
pub fn draw_icon(p: &mut Painter, x: i32, y: i32, icon_char: &str, label: &str, color: u32, selected: bool) {
    if selected {
        p.fill_rounded_rect(x - 6, y - 6, 60, 82, 6, 0x00D4A05640);
        p.draw_rect(x - 6, y - 6, 60, 82, colors::ACCENT);
    }

    // Icon box (48x48) with rounded corners
    p.fill_rounded_rect(x, y, 48, 48, 8, colors::ICON_BG);
    p.draw_rect(x, y, 48, 48, if selected { colors::ACCENT } else { colors::BORDER_DIM });

    // Icon letter (centered)
    p.draw_text(x + 20, y + 16, icon_char, color, colors::ICON_BG);

    // Label
    let lw = label.len() as i32 * 8;
    let lx = x + 24 - lw / 2;
    let bg = if selected { colors::DESKTOP_BG } else { colors::DESKTOP_BG };
    p.draw_text(lx, y + 54, label, colors::TEXT_DIM, bg);
}

// Helper for uppercase first char
trait ToUpperFirst {
    fn to_ascii_uppercase(&self) -> alloc::string::String;
}
impl ToUpperFirst for str {
    fn to_ascii_uppercase(&self) -> alloc::string::String {
        let mut s = alloc::string::String::from(self);
        if let Some(c) = s.get_mut(0..1) {
            c.make_ascii_uppercase();
        }
        s
    }
}
