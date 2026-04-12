use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use super::painter::Painter;
use super::window::Window;
use super::widgets::{self, colors};
use super::mouse::MOUSE;
use crate::arch::x86_64::idt::TICKS;
use core::sync::atomic::Ordering;

/// Execute a command in the GUI terminal. Returns lines of (text, color).
fn execute_terminal_cmd(cmd: &str) -> Vec<Option<(String, u32)>> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() { return Vec::new(); }
    let mut out = Vec::new();

    match parts[0] {
        "help" => {
            out.push(Some((String::from("Available commands:"), colors::TEXT_CYAN)));
            for &(cmd, desc) in &[
                ("help", "Show this help"),
                ("uname", "System information"),
                ("mem", "Memory usage"),
                ("lspci", "List PCI devices"),
                ("ps", "List threads"),
                ("ai", "Run AI inference"),
                ("uptime", "System uptime"),
                ("clear", "Clear terminal"),
                ("echo <text>", "Print text"),
                ("security", "Security status"),
                ("bench <N>", "Matrix benchmark"),
                ("neofetch", "System summary"),
            ] {
                out.push(Some((format!("  {:<14} {}", cmd, desc), colors::TEXT_DIM)));
            }
        }
        "uname" | "uname -a" => {
            out.push(Some((String::from("OpSys v0.1.0 x86_64 Microkernel (Rust)"), colors::TEXT_WHITE)));
        }
        "mem" | "free" => {
            let free = crate::mm::pmm::free_count();
            let mib = free * 4096 / 1024 / 1024;
            out.push(Some((format!("Free: {} MiB ({} frames)", mib, free), colors::TEXT_GREEN)));
        }
        "lspci" => {
            let devs = crate::drivers::pci::PCI_DEVICES.lock();
            for d in devs.iter() {
                out.push(Some((format!("{:02x}:{:02x}.{} {:04x}:{:04x} {}",
                    d.bus, d.device, d.function, d.vendor_id, d.device_id,
                    d.class_name()), colors::TEXT_DIM)));
            }
            out.push(Some((format!("{} device(s)", devs.len()), colors::TEXT_CYAN)));
        }
        "ps" => {
            out.push(Some((String::from("TID  STATE     PRI  NAME"), colors::TEXT_CYAN)));
            let sched = crate::proc::scheduler::SCHEDULER.lock();
            for tid in 0..16 {
                if let Some(t) = sched.thread_ref(tid) {
                    let st = match t.state {
                        crate::proc::thread::ThreadState::Running => "RUN",
                        crate::proc::thread::ThreadState::Ready => "RDY",
                        crate::proc::thread::ThreadState::Blocked => "BLK",
                        crate::proc::thread::ThreadState::Dead => "DED",
                        _ => "???",
                    };
                    out.push(Some((format!("{:<4} {:<9} {:<4} {}", t.tid, st, t.priority, t.name),
                        colors::TEXT_DIM)));
                }
            }
        }
        "ai" => {
            out.push(Some((String::from("Running MLP inference..."), colors::TEXT_PURPLE)));
            let session = crate::ai::runtime::InferenceSession::demo_mlp(8, 32, 4);
            let input = crate::ai::tensor::Tensor::from_f32(
                "in", &[1, 8], &[0.1, 0.5, 0.3, 0.8, 0.2, 0.6, 0.4, 0.9]);
            let output = session.forward(&input);
            let probs = output.as_f32();
            let mut max_i = 0;
            for (i, &p) in probs.iter().enumerate() { if p > probs[max_i] { max_i = i; } }
            out.push(Some((format!("Result: class {} ({:.1}%)", max_i, probs[max_i] * 100.0),
                colors::TEXT_YELLOW)));
        }
        "uptime" => {
            let ticks = TICKS.load(Ordering::Relaxed);
            let secs = ticks / 18;
            out.push(Some((format!("Up {}m {}s ({} ticks)", secs / 60, secs % 60, ticks),
                colors::TEXT_WHITE)));
        }
        "clear" => {
            return alloc::vec![None]; // Signal to clear
        }
        "echo" => {
            let text = parts[1..].join(" ");
            out.push(Some((text, colors::TEXT_WHITE)));
        }
        "security" => {
            let wx = if crate::security::wxe::is_enabled() { "ON" } else { "OFF" };
            let nx = if crate::security::cpu_protection::nx_enabled() { "ON" } else { "OFF" };
            out.push(Some((format!("W^X: {}  NX: {}  Capabilities: ON", wx, nx), colors::TEXT_GREEN)));
        }
        "bench" => {
            let n = parts.get(1).and_then(|s| s.parse::<usize>().ok()).unwrap_or(32);
            out.push(Some((format!("Benchmarking {}x{} matmul...", n, n), colors::TEXT_DIM)));
            let (ticks, _) = crate::ai::ops::bench_matmul(n);
            out.push(Some((format!("{} FLOPs in {} ticks", 2*n*n*n, ticks), colors::TEXT_YELLOW)));
        }
        "neofetch" => {
            let free = crate::mm::pmm::free_count() * 4096 / 1024 / 1024;
            let devs = crate::drivers::pci::PCI_DEVICES.lock().len();
            let ticks = TICKS.load(Ordering::Relaxed);
            out.push(Some((String::from("       ___  ___  ___"), colors::TEXT_ORANGE)));
            out.push(Some((String::from("      / _ \\/ _ \\/ __| OpSys v0.1.0"), colors::TEXT_ORANGE)));
            out.push(Some((String::from("     | (_) |  _/\\__ \\ ---------------"), colors::TEXT_ORANGE)));
            out.push(Some((format!    ("      \\___/|_|  |___/ Kernel: Microkernel (Rust)"), colors::TEXT_ORANGE)));
            out.push(Some((format!("                      Arch: x86_64"), colors::TEXT_DIM)));
            out.push(Some((format!("                      Memory: {} MiB free", free), colors::TEXT_DIM)));
            out.push(Some((format!("                      PCI: {} devices", devs), colors::TEXT_DIM)));
            out.push(Some((format!("                      Uptime: {}s", ticks / 18), colors::TEXT_DIM)));
            out.push(Some((format!("                      AI: Tensor engine active"), colors::TEXT_DIM)));
            out.push(Some((format!("                      Security: W^X + NX + Caps"), colors::TEXT_DIM)));
        }
        _ => {
            out.push(Some((format!("command not found: {}", parts[0]), colors::TEXT_RED)));
        }
    }
    out
}


const TASKBAR_H: i32 = 36;

/// Start menu items: (icon_key, label)
const MENU_ITEMS: &[(&str, &str)] = &[
    ("sys",  "System Info"),
    ("ai",   "AI Dashboard"),
    ("hw",   "Hardware"),
    ("set",  "Settings"),
    ("term", "Terminal Log"),
    ("pwr",  "Shutdown"),
];

/// Desktop icon definition.
struct DesktopIcon {
    x: i32,
    y: i32,
    icon: &'static str,
    label: &'static str,
    color: u32,
    action: &'static str, // which window to open
}

pub struct Desktop {
    painter: Painter,
    windows: Vec<Window>,
    next_id: usize,
    screen_w: i32,
    screen_h: i32,
    // Interaction state
    dragging: Option<(usize, i32, i32)>,
    start_menu_open: bool,
    start_hover_idx: Option<usize>,
    context_menu: Option<(i32, i32)>,
    selected_icon: Option<usize>,
    icons: Vec<DesktopIcon>,
    // Taskbar
    taskbar_window_btns: Vec<(i32, i32, usize, String)>,
    // Cursor tracking: save area under cursor for fast restore
    cursor_saved: [u32; 16 * 16],
    cursor_x: i32,
    cursor_y: i32,
    /// Full redraw needed (windows changed, menus opened, etc.)
    needs_full_redraw: bool,
}

impl Desktop {
    pub fn new(fb: *mut u8, width: u32, height: u32, pitch: u32) -> Self {
        let icons = alloc::vec![
            DesktopIcon { x: 30, y: 30, icon: "S", label: "System", color: colors::TEXT_CYAN, action: "sysinfo" },
            DesktopIcon { x: 30, y: 120, icon: "A", label: "AI", color: colors::TEXT_PURPLE, action: "ai" },
            DesktopIcon { x: 30, y: 210, icon: "H", label: "Hardware", color: colors::TEXT_GREEN, action: "hw" },
            DesktopIcon { x: 30, y: 300, icon: "G", label: "Settings", color: colors::TEXT_YELLOW, action: "settings" },
            DesktopIcon { x: 30, y: 390, icon: "T", label: "Terminal", color: colors::TEXT_WHITE, action: "terminal" },
        ];

        Self {
            painter: Painter::new(fb, width, height, pitch),
            windows: Vec::new(),
            next_id: 0,
            screen_w: width as i32,
            screen_h: height as i32,
            dragging: None,
            start_menu_open: false,
            start_hover_idx: None,
            context_menu: None,
            selected_icon: None,
            icons,
            taskbar_window_btns: Vec::new(),
            cursor_saved: [0; 16 * 16],
            cursor_x: -1,
            cursor_y: -1,
            needs_full_redraw: true,
        }
    }

    fn alloc_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn setup_default(&mut self) {
        self.open_window("sysinfo");
        self.open_window("ai");
        self.open_window("terminal");
        // Make terminal the active window so keyboard goes there
        for w in self.windows.iter_mut() {
            w.active = w.kind == "terminal";
        }
    }

    fn open_window(&mut self, kind: &str) {
        // Don't open duplicate windows
        for w in &self.windows {
            if w.kind == kind && w.visible { return; }
        }

        let id = self.alloc_id();
        let mut win = match kind {
            "sysinfo" => {
                let mut w = Window::new(id, "System Info", "sysinfo", 150, 50, 440, 340);
                w.add_line("OpSys v0.1.0 - AI-Optimized Microkernel");
                w.add_line("");
                w.add_styled("Architecture:", colors::TEXT_CYAN);
                w.add_line("  x86_64 (64-bit Long Mode)");
                w.add_styled("Kernel:", colors::TEXT_CYAN);
                w.add_line("  Microkernel written in Rust");
                w.add_styled("Security:", colors::TEXT_CYAN);
                w.add_line("  Capability-based + W^X + NX");
                w.add_line("");
                let free = crate::mm::pmm::free_count();
                w.add_styled(&format!("Memory: {} MiB free", free * 4096 / 1024 / 1024), colors::TEXT_GREEN);
                let devs = crate::drivers::pci::PCI_DEVICES.lock().len();
                w.add_line(&format!("PCI Devices: {}", devs));
                w.add_line(&format!("Display: {}x{}", self.screen_w, self.screen_h));
                w.add_line("");
                w.add_styled("All 7 phases complete", colors::TEXT_GREEN);
                w
            }
            "ai" => {
                let mut w = Window::new(id, "AI Dashboard", "ai", 620, 50, 460, 380);
                w.add_styled("AI Subsystem: ACTIVE", colors::TEXT_GREEN);
                w.add_line("");
                w.add_styled("Tensor Engine", colors::TEXT_CYAN);
                w.add_line("  Types: F32, F16, Q8_0, Q4_0");
                w.add_line("  Ops:   matmul, dot, relu, softmax");
                w.add_line("  Alloc: 64-byte aligned (SIMD-ready)");
                w.add_line("");
                w.add_styled("Model Runtime", colors::TEXT_CYAN);
                w.add_line("  Format: GGUF compatible");
                w.add_line("  Layers: Linear, ReLU, Softmax");
                w.add_line("");
                w.add_styled("Demo: MLP Classifier (8->32->4)", colors::TEXT_PURPLE);
                w.add_line("  Parameters: 420");
                w.add_line("  Memory:     1,680 bytes");
                // Run inference
                let session = crate::ai::runtime::InferenceSession::demo_mlp(8, 32, 4);
                let input = crate::ai::tensor::Tensor::from_f32(
                    "in", &[1, 8], &[0.1, 0.5, 0.3, 0.8, 0.2, 0.6, 0.4, 0.9]);
                let output = session.forward(&input);
                let probs = output.as_f32();
                let mut max_i = 0;
                for (i, &p) in probs.iter().enumerate() { if p > probs[max_i] { max_i = i; } }
                w.add_line("");
                w.add_styled(&format!("Inference: class {} ({:.1}%)", max_i, probs[max_i] * 100.0), colors::TEXT_YELLOW);
                w
            }
            "hw" => {
                let mut w = Window::new(id, "Hardware", "hw", 200, 300, 520, 280);
                w.add_styled("PCI Devices", colors::TEXT_CYAN);
                let devices = crate::drivers::pci::PCI_DEVICES.lock();
                for dev in devices.iter() {
                    w.add_line(&format!("  {:02x}:{:02x}.{} {:04x}:{:04x} {}",
                        dev.bus, dev.device, dev.function,
                        dev.vendor_id, dev.device_id, dev.class_name()));
                }
                drop(devices);
                w
            }
            "settings" => {
                let mut w = Window::new(id, "Settings", "settings", 300, 150, 460, 350);
                w.add_styled("System Settings", colors::TEXT_CYAN);
                w.add_line("");
                w.add_styled("Security", colors::TEXT_YELLOW);
                w.add_line(&format!("  W^X Enforcement:  {}",
                    if crate::security::wxe::is_enabled() { "ENABLED" } else { "DISABLED" }));
                w.add_line(&format!("  NX Bit:           {}",
                    if crate::security::cpu_protection::nx_enabled() { "ENABLED" } else { "DISABLED" }));
                w.add_line(&format!("  SMEP:             {}",
                    if crate::security::cpu_protection::smep_enabled() { "ENABLED" } else { "N/A" }));
                w.add_line(&format!("  SMAP:             {}",
                    if crate::security::cpu_protection::smap_enabled() { "ENABLED" } else { "N/A" }));
                w.add_line("");
                w.add_styled("Scheduler", colors::TEXT_YELLOW);
                w.add_line("  Priority Levels: 32");
                w.add_line("  AI Band:    24-31 (realtime)");
                w.add_line("  Normal:     16 (default)");
                w.add_line("  Idle:       0");
                w.add_line("");
                w.add_styled("IPC", colors::TEXT_YELLOW);
                w.add_line("  Endpoints:     synchronous (seL4-style)");
                w.add_line("  Notifications: async bitmask");
                w.add_line("  Messages:      64 bytes (8 registers)");
                w
            }
            "terminal" => {
                let mut w = Window::new(id, "Terminal", "terminal", 200, 200, 600, 380);
                w.is_terminal = true;
                w.add_styled("OpSys Terminal v0.1", colors::TEXT_CYAN);
                w.add_line("Type commands here. Try: help, uname, mem, lspci, ai");
                w.add_line("");
                w
            }
            _ => return,
        };

        // Make this window active
        for w in self.windows.iter_mut() { w.active = false; }
        win.active = true;
        self.windows.push(win);
    }

    pub fn handle_click(&mut self, mx: i32, my: i32) {
        self.context_menu = None;
        self.needs_full_redraw = true;

        let taskbar_y = self.screen_h - TASKBAR_H;

        // Start button click
        if my >= taskbar_y && mx >= 4 && mx < 94 {
            self.start_menu_open = !self.start_menu_open;
            return;
        }

        // Start menu item click
        if self.start_menu_open {
            let menu_x = 4;
            let menu_bottom = taskbar_y;
            let menu_h = MENU_ITEMS.len() as i32 * 32 + 16;
            let menu_top = menu_bottom - menu_h;

            if mx >= menu_x && mx < menu_x + 220 && my >= menu_top && my < menu_bottom {
                let idx = ((my - menu_top - 12) / 32) as usize;
                if idx < MENU_ITEMS.len() {
                    self.start_menu_open = false;
                    match MENU_ITEMS[idx].0 {
                        "pwr" => { /* shutdown - handled by shell */ }
                        key => self.open_window(match key {
                            "sys" => "sysinfo", "ai" => "ai", "hw" => "hw",
                            "set" => "settings", "term" => "terminal", _ => return,
                        }),
                    }
                }
                return;
            }
            self.start_menu_open = false;
        }

        // Taskbar window buttons
        for (bx, bw, wid, _) in &self.taskbar_window_btns {
            if my >= taskbar_y && mx >= *bx && mx < bx + bw {
                for w in self.windows.iter_mut() {
                    w.active = w.id == *wid;
                    if w.id == *wid { w.visible = true; }
                }
                return;
            }
        }

        // Desktop icons
        for (i, icon) in self.icons.iter().enumerate() {
            if mx >= icon.x - 4 && mx < icon.x + 52 && my >= icon.y - 4 && my < icon.y + 72 {
                self.selected_icon = Some(i);
                self.open_window(icon.action);
                return;
            }
        }
        self.selected_icon = None;

        // Windows (reverse order = topmost first)
        let mut clicked_id = None;
        for win in self.windows.iter().rev() {
            if !win.visible { continue; }
            if win.close_contains(mx, my) {
                let id = win.id;
                if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) { w.visible = false; }
                return;
            }
            if win.title_bar_contains(mx, my) {
                clicked_id = Some(win.id);
                self.dragging = Some((win.id, mx - win.x, my - win.y));
                break;
            }
            if win.contains(mx, my) {
                clicked_id = Some(win.id);
                break;
            }
        }
        if let Some(id) = clicked_id {
            for w in self.windows.iter_mut() { w.active = w.id == id; }
        }
    }

    pub fn handle_right_click(&mut self, mx: i32, my: i32) {
        self.start_menu_open = false;
        self.context_menu = Some((mx, my));
        self.needs_full_redraw = true;
    }

    pub fn handle_drag(&mut self, mx: i32, my: i32) {
        if let Some((wid, ox, oy)) = self.dragging {
            if let Some(win) = self.windows.iter_mut().find(|w| w.id == wid) {
                win.x = (mx - ox).clamp(0, self.screen_w - 100);
                win.y = (my - oy).clamp(0, self.screen_h - 60);
            }
            self.needs_full_redraw = true;
        }
    }

    pub fn handle_release(&mut self) { self.dragging = None; }

    /// Handle a keyboard character. Routes to the active terminal window.
    pub fn handle_key(&mut self, ch: u8) {
        // Find the active terminal window
        let active_term_id = self.windows.iter()
            .find(|w| w.active && w.visible && w.is_terminal)
            .map(|w| w.id);

        let Some(term_id) = active_term_id else { return; };
        self.needs_full_redraw = true;

        let term = self.windows.iter_mut().find(|w| w.id == term_id).unwrap();

        match ch {
            b'\n' => {
                // Execute the command
                let cmd = term.input_buf.clone();
                term.add_styled(&format!("$ {}", cmd), colors::TEXT_GREEN);
                term.input_buf.clear();
                // Execute and collect output
                let output = execute_terminal_cmd(&cmd);
                for line in output {
                    match line {
                        Some((text, color)) => term.add_styled(&text, color),
                        None => { term.lines.clear(); } // clear command
                    }
                }
            }
            0x08 => {
                // Backspace
                term.input_buf.pop();
            }
            0x03 => {
                // Ctrl+C
                term.input_buf.clear();
                term.add_line("^C");
            }
            0x1B => {} // Escape - ignore
            ch if ch >= 0x20 && ch <= 0x7E => {
                if term.input_buf.len() < 60 {
                    term.input_buf.push(ch as char);
                }
            }
            _ => {}
        }
    }

    pub fn handle_hover(&mut self, mx: i32, my: i32) {
        // Start menu hover
        if self.start_menu_open {
            let menu_bottom = self.screen_h - TASKBAR_H;
            let menu_h = MENU_ITEMS.len() as i32 * 32 + 16;
            let menu_top = menu_bottom - menu_h;
            if mx >= 4 && mx < 224 && my >= menu_top && my < menu_bottom {
                let idx = ((my - menu_top - 12) / 32) as usize;
                self.start_hover_idx = if idx < MENU_ITEMS.len() { Some(idx) } else { None };
            } else {
                self.start_hover_idx = None;
            }
        }
    }

    /// Fast cursor-only update: restore old cursor area, save new area, draw cursor.
    /// Only touches ~512 pixels instead of 1,024,000.
    pub fn update_cursor(&mut self) {
        let mouse = MOUSE.lock();
        let mx = mouse.x;
        let my = mouse.y;
        drop(mouse);

        // Restore old cursor area
        if self.cursor_x >= 0 {
            self.painter.restore_front_area(self.cursor_x, self.cursor_y, &self.cursor_saved);
        }

        // Save new cursor area
        self.painter.save_front_area(mx, my, &mut self.cursor_saved);

        // Draw cursor at new position
        self.painter.draw_cursor_front(mx, my);

        self.cursor_x = mx;
        self.cursor_y = my;
    }

    /// Full redraw: repaint everything to back buffer and flip.
    pub fn full_redraw(&mut self) {
        let p = &mut self.painter;
        let sw = self.screen_w;
        let sh = self.screen_h;

        // Desktop background
        p.fill_rect(0, 0, sw, sh, colors::DESKTOP_BG);

        // Subtle grid pattern
        for y in (0..sh).step_by(48) {
            for x in (0..sw).step_by(48) {
                p.put_pixel(x, y, 0x001A1A30);
            }
        }

        // Watermark
        p.draw_text_transparent(sw / 2 - 80, sh / 2 - 8, "OpSys Desktop", 0x001E1E38);

        // Desktop icons
        for (i, icon) in self.icons.iter().enumerate() {
            widgets::draw_icon(p, icon.x, icon.y, icon.icon, icon.label,
                icon.color, self.selected_icon == Some(i));
        }

        // Windows
        for win in &self.windows {
            if !win.visible { continue; }
            widgets::draw_window(p, win.x, win.y, win.width, win.height, &win.title, win.active);
            let cx = win.x + 10;
            let cy = win.y + 32;
            let max_y = win.y + win.height - 8;

            // Text content
            let mut last_y = cy;
            for (i, line) in win.lines.iter().enumerate() {
                let ly = cy + i as i32 * 16;
                if ly + 16 > max_y - if win.is_terminal { 20 } else { 0 } { break; }
                let color = line.color.unwrap_or(
                    if line.text.starts_with("  ") { colors::TEXT_DIM } else { colors::TEXT_WHITE }
                );
                p.draw_text(cx, ly, &line.text, color, colors::WINDOW_BG);
                last_y = ly + 16;
            }

            // Terminal input line
            if win.is_terminal {
                let input_y = win.y + win.height - 22;
                // Input separator
                p.fill_rect(win.x + 1, input_y - 2, win.width - 2, 1, colors::BORDER_DIM);
                // Prompt + input
                let prompt = "$ ";
                let display = format!("{}{}", prompt, win.input_buf);
                p.fill_rect(win.x + 1, input_y, win.width - 2, 18, 0x00161616);
                p.draw_text(cx, input_y + 1, &display, colors::TEXT_GREEN, 0x00161616);
                // Blinking cursor
                let ticks = TICKS.load(Ordering::Relaxed);
                if ticks % 18 < 9 {
                    let cursor_x = cx + (display.len() as i32) * 8;
                    p.fill_rect(cursor_x, input_y + 1, 8, 14, colors::TEXT_GREEN);
                }
            }
        }

        // Taskbar
        let taskbar_y = sh - TASKBAR_H;
        let mouse = MOUSE.lock();
        let start_hover = mouse.y >= taskbar_y && mouse.x >= 4 && mouse.x < 94;
        drop(mouse);

        widgets::draw_taskbar(p, taskbar_y, sw, start_hover);

        // Taskbar window buttons
        self.taskbar_window_btns.clear();
        let mut btn_x = 100;
        for win in &self.windows {
            if !win.visible { continue; }
            let title: String = if win.title.len() > 15 {
                format!("{}...", &win.title[..12])
            } else { win.title.clone() };
            let btn_w = (title.len() as i32 + 2) * 8;
            let bg = if win.active { colors::TITLE_ACTIVE } else { colors::BTN_BG };
            p.fill_rect(btn_x, taskbar_y + 5, btn_w, 26, bg);
            p.draw_rect(btn_x, taskbar_y + 5, btn_w, 26, colors::BORDER_DIM);
            if win.active { p.fill_rect(btn_x, taskbar_y + 29, btn_w, 2, colors::ACCENT); }
            p.draw_text(btn_x + 8, taskbar_y + 10, &title, colors::TEXT_WHITE, bg);
            self.taskbar_window_btns.push((btn_x, btn_w, win.id, title));
            btn_x += btn_w + 4;
        }

        // Clock
        let ticks = TICKS.load(Ordering::Relaxed);
        let secs = ticks / 18;
        let time_str = format!("{:02}:{:02}:{:02}", (secs / 3600) % 24, (secs / 60) % 60, secs % 60);
        let tx = sw - (time_str.len() as i32 + 2) * 8;
        p.draw_text(tx, taskbar_y + 10, &time_str, colors::TEXT_WHITE, colors::TASKBAR_BG);

        // Start menu
        if self.start_menu_open {
            widgets::draw_start_menu(p, 4, taskbar_y, MENU_ITEMS, self.start_hover_idx);
        }

        // Context menu
        if let Some((cmx, cmy)) = self.context_menu {
            let items = ["Refresh", "Settings", "About OpSys"];
            let mw = 160;
            let mh = items.len() as i32 * 28 + 8;
            p.fill_rounded_rect(cmx, cmy, mw, mh, 3, colors::MENU_BG);
            p.draw_rect(cmx, cmy, mw, mh, colors::BORDER_BRIGHT);
            for (i, item) in items.iter().enumerate() {
                p.draw_text(cmx + 12, cmy + 4 + i as i32 * 28 + 6, item, colors::TEXT_WHITE, colors::MENU_BG);
            }
        }

        // Flip back buffer to front (WITHOUT cursor — cursor is drawn directly on front)
        self.painter.flip();

        // Now draw cursor on the front buffer using the fast path
        let mouse = MOUSE.lock();
        self.cursor_x = mouse.x;
        self.cursor_y = mouse.y;
        drop(mouse);
        // Save the clean area BEFORE drawing cursor on it
        self.painter.save_front_area(self.cursor_x, self.cursor_y, &mut self.cursor_saved);
        // Draw cursor on front buffer
        self.painter.draw_cursor_front(self.cursor_x, self.cursor_y);

        self.needs_full_redraw = false;
    }

    /// Main draw entry: full redraw if needed, otherwise just move the cursor.
    pub fn draw(&mut self) {
        if self.needs_full_redraw {
            self.full_redraw();
        } else {
            self.update_cursor();
        }
    }
}
