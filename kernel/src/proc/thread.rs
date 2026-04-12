use alloc::boxed::Box;
use core::sync::atomic::{AtomicUsize, Ordering};

/// Thread ID counter. TID 0 is reserved for the boot thread.
static NEXT_TID: AtomicUsize = AtomicUsize::new(1);

/// Size of a kernel thread stack (16 KiB).
pub const KERNEL_STACK_SIZE: usize = 4096 * 4;

/// Thread states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadState {
    Ready,
    Running,
    Blocked,
    Sleeping,
    Dead,
}

/// Saved CPU register context for context switching.
/// Only callee-saved registers + stack pointer + instruction pointer.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Context {
    pub rbx: u64,
    pub rbp: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rsp: u64,
    pub rip: u64,
    pub rflags: u64,
}

impl Context {
    pub const fn empty() -> Self {
        Self {
            rbx: 0, rbp: 0, r12: 0, r13: 0, r14: 0, r15: 0,
            rsp: 0, rip: 0, rflags: 0x200, // IF=1 (interrupts enabled)
        }
    }
}

/// A kernel thread.
pub struct Thread {
    pub tid: usize,
    pub pid: usize,
    pub state: ThreadState,
    pub priority: u8,
    pub context: Context,
    /// Kernel stack (owned allocation). None for the boot thread.
    _stack: Option<Box<[u8; KERNEL_STACK_SIZE]>>,
    pub name: &'static str,
}

impl Thread {
    /// Create the boot thread (TID 0).
    /// Represents the current execution context (kernel_main).
    /// No stack is allocated — it's already on the Limine kernel stack.
    /// The context will be filled in by the first context switch.
    pub fn boot_thread() -> Self {
        Self {
            tid: 0,
            pid: 0,
            state: ThreadState::Running,
            priority: 0, // Lowest priority — yields to everything
            context: Context::empty(),
            _stack: None,
            name: "boot",
        }
    }

    /// Create a new kernel thread that will execute `entry_point`.
    pub fn new_kernel(
        pid: usize,
        priority: u8,
        name: &'static str,
        entry_point: fn(),
    ) -> Self {
        let tid = NEXT_TID.fetch_add(1, Ordering::Relaxed);

        // Allocate a kernel stack
        let stack = Box::new([0u8; KERNEL_STACK_SIZE]);
        let stack_top = stack.as_ptr() as u64 + KERNEL_STACK_SIZE as u64;

        // Set up the initial context:
        // - rsp points to the top of the stack (minus 8 for alignment)
        // - rip is set to entry_point
        // When the context switch does `jmp [rsi + 0x38]`, it jumps to entry_point.
        let mut context = Context::empty();
        context.rsp = stack_top - 8; // 16-byte aligned after the jmp pushes return addr
        context.rip = entry_point as u64;
        context.rflags = 0x200; // Interrupts enabled

        Self {
            tid,
            pid,
            state: ThreadState::Ready,
            priority,
            context,
            _stack: Some(stack),
            name,
        }
    }
}
