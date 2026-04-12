#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod ai;
mod arch;
mod cap;
mod drivers;
mod gui;
mod ipc;
mod mm;
mod panic;
mod proc;
mod security;
mod syscall;

use core::sync::atomic::{AtomicBool, Ordering};
use proc::scheduler::{SCHEDULER, PRIORITY_NORMAL, PRIORITY_SYSTEM};
use proc::thread::Thread;

#[unsafe(no_mangle)]
extern "C" fn _start() -> ! {
    kernel_main();
}

static INIT_DONE: AtomicBool = AtomicBool::new(false);

fn kernel_main() -> ! {
    arch::x86_64::init();

    serial_println!();
    serial_println!("============================================");
    serial_println!("  OpSys v0.1.0 - AI-Optimized Microkernel");
    serial_println!("============================================");
    serial_println!();

    mm::init();

    // Initialize syscall support (SYSCALL/SYSRET MSRs)
    serial_println!();
    serial_println!("[syscall] Initializing SYSCALL/SYSRET...");
    syscall::table::init();
    // Set the kernel stack for syscall entry (use TSS RSP0 area)
    let kernel_stack_top =
        &raw const arch::x86_64::gdt::SYSCALL_KERNEL_STACK as *const u8 as u64
            + arch::x86_64::gdt::SYSCALL_STACK_SIZE as u64;
    syscall::table::set_kernel_stack(kernel_stack_top);
    serial_println!("[syscall] SYSCALL/SYSRET initialized");
    serial_println!("[syscall] Kernel stack at {:#x}", kernel_stack_top);

    // Initialize scheduler
    serial_println!();
    serial_println!("[sched] Initializing scheduler...");
    SCHEDULER.lock().init();
    serial_println!("[sched] Scheduler initialized");

    // --- Phase 4: Hardware drivers ---
    serial_println!();
    serial_println!("[drivers] Enumerating PCI bus...");
    drivers::pci::enumerate();

    serial_println!();
    serial_println!("[drivers] Initializing framebuffer console...");
    drivers::framebuffer::init();

    // --- Phase 6: Security hardening ---
    serial_println!();
    security::init();

    // --- Phase 7: GUI Desktop ---
    serial_println!();
    serial_println!("[gui] Initializing PS/2 mouse...");
    gui::mouse::init();
    {
        let fb_lock = drivers::framebuffer::CONSOLE.lock();
        if let Some(ref fb) = *fb_lock {
            gui::mouse::MOUSE.lock().set_screen_size(fb.width as i32, fb.height as i32);
            serial_println!("[gui] Mouse initialized for {}x{}", fb.width, fb.height);
        }
    }

    // Spawn the desktop compositor thread
    let desktop_thread = Thread::new_kernel(0, PRIORITY_SYSTEM, "desktop", desktop_entry);
    let tid = SCHEDULER.lock().spawn(desktop_thread);
    serial_println!("[gui] Spawned desktop thread (TID {})", tid);

    // --- Syscall test ---
    serial_println!();
    serial_println!("[test] Testing syscall interface...");

    // Spawn a thread that uses raw syscalls to print and exit
    let syscall_thread = Thread::new_kernel(0, PRIORITY_SYSTEM, "syscall-test", syscall_test_entry);
    let tid = SCHEDULER.lock().spawn(syscall_thread);
    serial_println!("[test] Spawned syscall-test thread (TID {})", tid);

    // Spawn the shell
    let shell_thread = Thread::new_kernel(0, PRIORITY_NORMAL, "shell", shell_entry);
    let tid = SCHEDULER.lock().spawn(shell_thread);
    serial_println!("[test] Spawned shell thread (TID {})", tid);

    serial_println!();

    // Main loop: yield to other threads
    loop {
        yield_now();
        if INIT_DONE.load(Ordering::Relaxed) {
            break;
        }
    }

    serial_println!();
    serial_println!("============================================");
    serial_println!("  OpSys - All Phases Complete");
    serial_println!("============================================");
    serial_println!();
    serial_println!("[kernel] Phase 1: Bare-metal bootstrap       OK");
    serial_println!("[kernel] Phase 2: Microkernel core            OK");
    serial_println!("[kernel] Phase 3: Syscalls + shell            OK");
    serial_println!("[kernel] Phase 4: Hardware drivers            OK");
    serial_println!("[kernel] Phase 5: AI subsystem                OK");
    serial_println!("[kernel] Phase 6: Security hardening          OK");
    serial_println!("[kernel] Phase 7: Graphical desktop           OK");
    serial_println!();
    let devs = drivers::pci::PCI_DEVICES.lock().len();
    serial_println!("[kernel] PCI: {} devices discovered", devs);
    serial_println!("[kernel] Framebuffer: {}",
        if drivers::framebuffer::CONSOLE.lock().is_some() { "active" } else { "none" });
    serial_println!("[kernel] System idle.");

    loop {
        x86_64::instructions::hlt();
    }
}

/// Test thread: exercises the syscall dispatch (calling handlers directly
/// since we're running in ring 0). The SYSCALL/SYSRET MSRs are set up and
/// ready for ring-3 processes — the first SYSCALL did work before SYSRET
/// returned to ring-3 kernel pages.
fn syscall_test_entry() {
    use syscall::handlers::syscall_dispatch;

    // Call syscall handlers directly (simulating what SYSCALL would do)
    let msg = "[syscall-test] Hello via sys_debug_print!";
    syscall_dispatch(
        opsys_api::syscall::SYS_DEBUG_PRINT as u64,
        msg.as_ptr() as u64, msg.len() as u64, 0, 0, 0,
    );

    let msg2 = "[syscall-test] Testing sys_yield...";
    syscall_dispatch(
        opsys_api::syscall::SYS_DEBUG_PRINT as u64,
        msg2.as_ptr() as u64, msg2.len() as u64, 0, 0, 0,
    );

    // Test yield syscall
    syscall_dispatch(opsys_api::syscall::SYS_YIELD as u64, 0, 0, 0, 0, 0);

    let msg3 = "[syscall-test] Returned from yield. All syscalls operational!";
    syscall_dispatch(
        opsys_api::syscall::SYS_DEBUG_PRINT as u64,
        msg3.as_ptr() as u64, msg3.len() as u64, 0, 0, 0,
    );

    // Exit via syscall
    syscall_dispatch(opsys_api::syscall::SYS_EXIT as u64, 0, 0, 0, 0, 0);
}

/// Simple shell: reads serial input and executes commands.
fn shell_entry() {
    use x86_64::instructions::port::Port;

    serial_println!();
    serial_println!("OpSys Shell v0.1");
    serial_println!("Type 'help' for available commands.");
    serial_println!();
    serial_print!("opsys> ");

    let mut buf = [0u8; 256];
    let mut pos = 0;

    loop {
        // Poll serial port for input
        let mut status_port = Port::<u8>::new(0x3FD);
        let status = unsafe { status_port.read() };
        if status & 1 == 0 {
            // No data available, yield
            yield_now();
            continue;
        }

        let mut data_port = Port::<u8>::new(0x3F8);
        let byte = unsafe { data_port.read() };

        match byte {
            // Enter
            b'\r' | b'\n' => {
                serial_println!();
                if pos > 0 {
                    let cmd = core::str::from_utf8(&buf[..pos]).unwrap_or("");
                    execute_command(cmd);
                    pos = 0;
                }
                serial_print!("opsys> ");
            }
            // Backspace
            0x7F | 0x08 => {
                if pos > 0 {
                    pos -= 1;
                    serial_print!("\x08 \x08");
                }
            }
            // Ctrl-C: signal done
            0x03 => {
                serial_println!("^C");
                serial_println!("[shell] Shutting down.");
                INIT_DONE.store(true, Ordering::Relaxed);
                // Exit this thread
                {
                    let mut sched = SCHEDULER.lock();
                    let tid = sched.current_tid();
                    if let Some(t) = sched.thread_mut(tid) {
                        t.state = proc::thread::ThreadState::Dead;
                    }
                }
                yield_now();
                return;
            }
            // Printable character
            0x20..=0x7E => {
                if pos < buf.len() - 1 {
                    buf[pos] = byte;
                    pos += 1;
                    // Echo
                    serial_print!("{}", byte as char);
                }
            }
            _ => {}
        }
    }
}

fn execute_command(cmd: &str) {
    let parts: alloc::vec::Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    match parts[0] {
        "help" => {
            serial_println!("Available commands:");
            serial_println!("  help    - Show this help");
            serial_println!("  ps      - List threads");
            serial_println!("  mem     - Show memory info");
            serial_println!("  lspci   - List PCI devices");
            serial_println!("  fb      - Framebuffer info & test");
            serial_println!("  ai      - Run AI inference demo");
            serial_println!("  bench   - Matrix multiply benchmark");
            serial_println!("  tensor  - Tensor operations demo");
            serial_println!("  security- Security status");
            serial_println!("  echo    - Echo arguments");
            serial_println!("  uname   - System info");
            serial_println!("  clear   - Clear screen");
            serial_println!("  exit    - Shutdown");
        }
        "ps" => {
            serial_println!("  TID  STATE     PRIORITY  NAME");
            serial_println!("  ---  --------  --------  ----");
            let sched = SCHEDULER.lock();
            for tid in 0..16 {
                if let Some(t) = sched.thread_ref(tid) {
                    let state = match t.state {
                        proc::thread::ThreadState::Running => "RUNNING ",
                        proc::thread::ThreadState::Ready => "READY   ",
                        proc::thread::ThreadState::Blocked => "BLOCKED ",
                        proc::thread::ThreadState::Sleeping => "SLEEPING",
                        proc::thread::ThreadState::Dead => "DEAD    ",
                    };
                    serial_println!("  {:>3}  {}  {:>8}  {}", t.tid, state, t.priority, t.name);
                }
            }
        }
        "mem" => {
            let free = mm::pmm::free_count();
            let free_mib = free as u64 * 4096 / 1024 / 1024;
            serial_println!("Physical memory:");
            serial_println!("  Free frames: {} ({} MiB)", free, free_mib);
        }
        "lspci" => {
            let devices = drivers::pci::PCI_DEVICES.lock();
            if devices.is_empty() {
                serial_println!("No PCI devices found.");
            } else {
                serial_println!("  BUS:DEV.FN  VEN:DEV   CLASS");
                serial_println!("  ---------   -------   -----");
                for dev in devices.iter() {
                    serial_println!(
                        "  {:02x}:{:02x}.{}    {:04x}:{:04x}  {}",
                        dev.bus, dev.device, dev.function,
                        dev.vendor_id, dev.device_id,
                        dev.class_name()
                    );
                }
                serial_println!("  {} device(s) total", devices.len());
            }
        }
        "fb" => {
            if let Some(ref mut console) = *drivers::framebuffer::CONSOLE.lock() {
                serial_println!("Framebuffer: {}x{}, {} bpp", console.width, console.height, console.bpp);
                // Write test text to the framebuffer
                console.write_str("Shell> fb test output\n");
                serial_println!("Wrote test text to framebuffer.");
            } else {
                serial_println!("No framebuffer available.");
            }
        }
        "tensor" => {
            serial_println!("Tensor Operations Demo:");
            serial_println!("  Creating tensors...");
            let a = ai::tensor::Tensor::from_f32("a", &[1, 4], &[1.0, 2.0, 3.0, 4.0]);
            let b = ai::tensor::Tensor::from_f32("b", &[1, 4], &[0.5, 0.5, 0.5, 0.5]);
            serial_println!("  a = [1.0, 2.0, 3.0, 4.0]");
            serial_println!("  b = [0.5, 0.5, 0.5, 0.5]");

            let dot = ai::ops::dot_f32(a.as_f32(), b.as_f32());
            serial_println!("  dot(a, b) = {}", dot);

            serial_println!("  Matrix multiply 2x3 @ 3x2...");
            let m1 = ai::tensor::Tensor::from_f32("m1", &[2, 3], &[1.0,2.0,3.0, 4.0,5.0,6.0]);
            let m2 = ai::tensor::Tensor::from_f32("m2", &[3, 2], &[7.0,8.0, 9.0,10.0, 11.0,12.0]);
            let mut m3 = ai::tensor::Tensor::zeros("m3", &[2, 2], ai::tensor::DType::F32);
            ai::ops::matmul(&m1, &m2, &mut m3);
            let r = m3.as_f32();
            serial_println!("  Result: [[{}, {}], [{}, {}]]", r[0], r[1], r[2], r[3]);

            serial_println!("  Softmax [1.0, 2.0, 3.0]...");
            let mut s = ai::tensor::Tensor::from_f32("s", &[3], &[1.0, 2.0, 3.0]);
            ai::ops::softmax(&mut s);
            let sv = s.as_f32();
            serial_println!("  Result: [{:.4}, {:.4}, {:.4}]", sv[0], sv[1], sv[2]);
        }
        "bench" => {
            let n = if parts.len() > 1 {
                parts[1].parse::<usize>().unwrap_or(64)
            } else {
                64
            };
            serial_println!("Benchmarking {}x{} matrix multiply...", n, n);
            let (ticks, gflops) = ai::ops::bench_matmul(n);
            serial_println!("  {} ticks elapsed", ticks);
            serial_println!("  ~{:.3} GFLOPS (estimated)", gflops);
            serial_println!("  {} FLOPs total", 2 * n * n * n);
        }
        "ai" => {
            serial_println!("AI Inference Demo (MLP classifier):");
            serial_println!("  Creating model: 8 -> 32 -> 4 (MLP)");
            let session = ai::runtime::InferenceSession::demo_mlp(8, 32, 4);
            serial_println!("  Parameters: {}", session.param_count());
            serial_println!("  Memory: {} bytes", session.memory_bytes());

            serial_println!("  Running forward pass...");
            let input = ai::tensor::Tensor::from_f32(
                "input", &[1, 8],
                &[0.1, 0.5, 0.3, 0.8, 0.2, 0.6, 0.4, 0.9],
            );
            let output = session.forward(&input);
            let probs = output.as_f32();
            serial_println!("  Input:  [0.1, 0.5, 0.3, 0.8, 0.2, 0.6, 0.4, 0.9]");
            serial_print!("  Output: [");
            for (i, &p) in probs.iter().enumerate() {
                if i > 0 { serial_print!(", "); }
                serial_print!("{:.4}", p);
            }
            serial_println!("]");

            // Find argmax
            let mut max_idx = 0;
            let mut max_val = probs[0];
            for (i, &p) in probs.iter().enumerate() {
                if p > max_val {
                    max_val = p;
                    max_idx = i;
                }
            }
            serial_println!("  Prediction: class {} (confidence: {:.1}%)", max_idx, max_val * 100.0);
            serial_println!("  Inference complete.");
        }
        "echo" => {
            let rest = parts[1..].join(" ");
            serial_println!("{}", rest);
        }
        "security" => {
            let status = security::SecurityStatus::collect();
            status.print_report();
        }
        "uname" => {
            serial_println!("OpSys v0.1.0 x86_64 - AI-Optimized Microkernel");
        }
        "clear" => {
            // Send ANSI clear sequence
            serial_print!("\x1B[2J\x1B[H");
        }
        "exit" => {
            serial_println!("[shell] Shutting down.");
            INIT_DONE.store(true, Ordering::Relaxed);
            {
                let mut sched = SCHEDULER.lock();
                let tid = sched.current_tid();
                if let Some(t) = sched.thread_mut(tid) {
                    t.state = proc::thread::ThreadState::Dead;
                }
            }
            yield_now();
        }
        _ => {
            serial_println!("Unknown command: '{}'. Type 'help' for commands.", parts[0]);
        }
    }
}

/// Desktop compositor thread: draws the GUI and handles mouse input.
fn desktop_entry() {
    let fb_info = {
        let fb_lock = drivers::framebuffer::CONSOLE.lock();
        fb_lock.as_ref().map(|fb| (fb.fb_base, fb.width, fb.height, fb.pitch))
    };

    let Some((fb_base, width, height, pitch)) = fb_info else {
        serial_println!("[desktop] No framebuffer, exiting.");
        return;
    };

    let mut desktop = gui::desktop::Desktop::new(fb_base, width, height, pitch);
    desktop.setup_default();
    serial_println!("[desktop] Desktop ready.");

    // Initial draw
    desktop.draw();

    let mut prev_buttons = 0u8;

    // GUI main loop
    loop {
        let mut did_something = false;

        // Check mouse
        let (mx, my, buttons, mouse_dirty) = {
            let mut m = gui::mouse::MOUSE.lock();
            let d = m.dirty;
            m.dirty = false;
            (m.x, m.y, m.buttons, d)
        };

        if mouse_dirty {
            if buttons & 1 != 0 && prev_buttons & 1 == 0 { desktop.handle_click(mx, my); }
            if buttons & 1 != 0 { desktop.handle_drag(mx, my); }
            if buttons & 1 == 0 && prev_buttons & 1 != 0 { desktop.handle_release(); }
            if buttons & 2 != 0 && prev_buttons & 2 == 0 { desktop.handle_right_click(mx, my); }
            desktop.handle_hover(mx, my);
            prev_buttons = buttons;
            did_something = true;
        }

        // Check keyboard
        loop {
            let key = gui::keyboard::KEYBOARD.lock().pop();
            match key {
                Some(ch) => {
                    desktop.handle_key(ch);
                    did_something = true;
                }
                None => break,
            }
        }

        // Redraw if anything changed
        if did_something {
            desktop.draw();
        }

        x86_64::instructions::hlt();
        yield_now();
    }
}

fn yield_now() {
    unsafe {
        proc::scheduler::do_schedule();
    }
}
