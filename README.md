# OpSys — AI-Optimized Microkernel Operating System

A lightweight, secure operating system built from scratch in Rust, designed for AI workloads.

## Features

- **Microkernel architecture** — minimal kernel, drivers in userspace
- **Capability-based security** — unforgeable tokens for all resource access (seL4-inspired)
- **AI inference engine** — built-in tensor operations, MLP inference, GGUF model support
- **Graphical desktop** — window manager, mouse input, taskbar, themed UI
- **32-level priority scheduler** — AI workloads get dedicated priority bands
- **Hardware discovery** — full PCI bus enumeration
- **Interactive shell** — 12 built-in commands

## Screenshots

Boot OpSys and you'll see a graphical desktop with:
- **System Info** window — architecture, memory, security status
- **AI Dashboard** — tensor engine status, live inference results
- **Hardware** window — PCI device listing
- **Taskbar** with OpSys button and tick counter

## Quick Start

### Prerequisites

```bash
# Ubuntu/Debian
sudo apt-get install -y qemu-system-x86 xorriso mtools git build-essential

# Install Rust nightly
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly
source ~/.cargo/env
rustup component add rust-src llvm-tools-preview
```

### Build & Run

```bash
git clone https://github.com/YOUR_USERNAME/opsys.git
cd opsys
make run
```

This builds the kernel, creates a bootable ISO, and launches QEMU with:
- Graphical display (the desktop)
- Serial console on stdio (the shell)

### Shell Commands

| Command | Description |
|---------|-------------|
| `help` | List all commands |
| `uname` | System info |
| `ps` | List threads with state and priority |
| `mem` | Physical memory usage |
| `lspci` | List PCI devices |
| `fb` | Framebuffer info |
| `ai` | Run AI inference demo |
| `bench N` | NxN matrix multiply benchmark |
| `tensor` | Tensor operations demo |
| `security` | Security feature status |
| `echo TEXT` | Echo text |
| `exit` | Shutdown |

### QEMU Options

```bash
# Standard (graphical + serial shell)
make run

# With KVM acceleration (faster, uses host CPU features)
make run-kvm

# Debug mode (GDB server on localhost:1234)
make debug

# In another terminal:
gdb -ex "target remote :1234" -ex "symbol-file target/x86_64-unknown-none/debug/opsys-kernel"
```

## Deploy to Real Hardware

### Create Bootable USB

```bash
make iso-release
sudo ./tools/make-usb.sh /dev/sdX    # Replace sdX with your USB drive
```

### Boot from USB

1. Plug the USB into your target machine
2. Enter BIOS/UEFI boot menu (F12, F2, or Del)
3. Select the USB drive
4. OpSys boots with the graphical desktop

**Note:** Serial output goes to COM1 at 115200 baud. Connect a serial cable or use a USB-to-serial adapter to access the shell on real hardware.

### Boot from ISO (Virtual Machine)

The ISO at `build/opsys.iso` can be used directly with:
- **QEMU** — `qemu-system-x86_64 -cdrom build/opsys.iso -m 256M`
- **VirtualBox** — Create a new VM, attach the ISO as a CD
- **VMware** — Create a new VM, use the ISO as boot media

## Architecture

```
kernel/src/
├── arch/x86_64/     Boot, GDT, IDT, PIC, serial, CPU feature detection
├── mm/              Physical memory (bitmap), virtual memory (HHDM), heap
├── proc/            Threads, 32-priority scheduler, context switching
├── cap/             Capability tokens, CSpace, rights, kernel objects
├── ipc/             Synchronous endpoints, async notifications, messages
├── syscall/         SYSCALL/SYSRET, dispatch table, handlers
├── drivers/
│   ├── pci/         PCI bus enumeration, BAR parsing, class identification
│   └── framebuffer/ Pixel rendering, 8x16 bitmap font (95 ASCII glyphs)
├── ai/              Tensors, matmul, softmax, GGUF parser, MLP inference
├── security/        W^X enforcement, SMEP/SMAP, NX bit
├── gui/
│   ├── mouse.rs     PS/2 driver (IRQ12), packet decode
│   ├── painter.rs   Pixel/rect/text/cursor drawing
│   ├── widgets.rs   Window frames, panels, taskbar, color theme
│   ├── window.rs    Window management, hit testing, dragging
│   └── desktop.rs   Compositor with gradient bg, windows, taskbar
└── main.rs          Kernel entry + interactive shell
```

### Design Decisions

| Decision | Rationale |
|----------|-----------|
| Rust | Memory safety without GC — prevents entire classes of kernel bugs |
| Microkernel | Driver crashes can't take down the kernel |
| Capability-based security | No ambient authority — every access needs an unforgeable token |
| Limine bootloader | Handles UEFI/BIOS, saves weeks of bootloader development |
| 32-priority scheduler | Dedicated bands for AI workloads (24-31) vs normal tasks (16) |
| Register-based IPC | Messages ≤64 bytes pass in registers — zero-copy fast path |

## Build Requirements

- Rust nightly (with `rust-src`, `llvm-tools-preview`)
- QEMU (`qemu-system-x86_64`)
- `xorriso` and `mtools` (for ISO creation)
- `git` and `build-essential`

## License

MIT
