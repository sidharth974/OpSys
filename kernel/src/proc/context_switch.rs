use super::thread::Context;

/// Switch from the current thread's context to a new thread's context.
///
/// Saves callee-saved registers into `old`, then restores from `new`.
/// The `ret` at the end jumps to `new.rip` (the saved return address on the new stack).
///
/// # Safety
/// Both pointers must be valid Context structs. The new context must have
/// a valid stack and return address.
#[inline(never)]
pub unsafe fn switch_context(old: *mut Context, new: *const Context) {
    // Save current callee-saved registers into `old`
    // Restore callee-saved registers from `new`
    // Switch stack pointers
    // `ret` will pop the return address from the new stack and jump there
    unsafe { core::arch::asm!(
        // Save callee-saved registers to old context
        "mov [rdi + 0x00], rbx",
        "mov [rdi + 0x08], rbp",
        "mov [rdi + 0x10], r12",
        "mov [rdi + 0x18], r13",
        "mov [rdi + 0x20], r14",
        "mov [rdi + 0x28], r15",
        "mov [rdi + 0x30], rsp",
        // Save return address (the address after this asm block)
        "lea rax, [rip + 2f]",
        "mov [rdi + 0x38], rax",
        // Save rflags
        "pushfq",
        "pop rax",
        "mov [rdi + 0x40], rax",

        // Restore callee-saved registers from new context
        "mov rbx, [rsi + 0x00]",
        "mov rbp, [rsi + 0x08]",
        "mov r12, [rsi + 0x10]",
        "mov r13, [rsi + 0x18]",
        "mov r14, [rsi + 0x20]",
        "mov r15, [rsi + 0x28]",
        "mov rsp, [rsi + 0x30]",

        // Restore rflags
        "mov rax, [rsi + 0x40]",
        "push rax",
        "popfq",

        // Jump to the new thread's saved instruction pointer.
        // For a brand-new thread, this is entry_point.
        // For a previously-running thread, this is the label "2:" below.
        "jmp [rsi + 0x38]",

        // This is where a resumed thread continues from.
        "2:",

        in("rdi") old,
        in("rsi") new,
        // Clobber all caller-saved registers
        out("rax") _,
        out("rcx") _,
        out("rdx") _,
        // r8-r11 are caller-saved
        out("r8") _,
        out("r9") _,
        out("r10") _,
        out("r11") _,
        clobber_abi("C"),
    ); }
}
