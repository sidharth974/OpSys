use x86_64::registers::model_specific::{Efer, EferFlags, Star, LStar, SFMask};
use x86_64::registers::rflags::RFlags;
use x86_64::VirtAddr;
use crate::arch::x86_64::gdt;

/// Initialize SYSCALL/SYSRET support.
/// After this, userspace can execute `syscall` to enter the kernel.
pub fn init() {
    let selectors = gdt::selectors();

    unsafe {
        // Enable SCE (System Call Extensions) in EFER MSR
        Efer::update(|flags| *flags |= EferFlags::SYSTEM_CALL_EXTENSIONS);

        // STAR MSR: sets the CS/SS selectors for SYSCALL and SYSRET.
        // SYSCALL loads CS from bits 32-47, SS = CS + 8
        // SYSRET loads CS from bits 48-63 + 16, SS from bits 48-63 + 8
        Star::write(
            selectors.user_code,
            selectors.user_data,
            selectors.kernel_code,
            selectors.kernel_data,
        ).expect("Failed to write STAR MSR");

        // LSTAR MSR: the kernel entry point for SYSCALL
        LStar::write(VirtAddr::new(syscall_entry_naked as u64));

        // SFMASK MSR: RFLAGS bits to clear on SYSCALL entry.
        // Clear IF (disable interrupts) and TF (no single-step) on entry.
        SFMask::write(RFlags::INTERRUPT_FLAG | RFlags::TRAP_FLAG);
    }
}

/// Naked syscall entry point. This is where `SYSCALL` lands.
///
/// On entry (from SYSCALL instruction):
///   RCX = user RIP (return address)
///   R11 = user RFLAGS
///   RAX = syscall number
///   RDI, RSI, RDX, R10, R8, R9 = arguments (Linux convention)
///   RSP = still user RSP (SYSCALL does NOT switch stacks!)
///
/// We must:
///   1. Switch to the kernel stack
///   2. Save user RSP
///   3. Call the Rust syscall dispatcher
///   4. Restore user RSP
///   5. SYSRETQ back to userspace
#[unsafe(naked)]
extern "C" fn syscall_entry_naked() {
    core::arch::naked_asm!(
        // Swap to kernel stack. We save user RSP in a scratch register.
        "mov r15, rsp",         // Save user RSP in r15 temporarily

        // Load kernel RSP from the TSS RSP0.
        // We use gs:0 or a fixed location. For simplicity, we use
        // a global variable for the kernel stack pointer.
        "lea rsp, [{kernel_stack_top}]",
        "mov rsp, [rsp]",

        // Build a syscall frame on the kernel stack
        "push r15",             // User RSP
        "push r11",             // User RFLAGS (saved by SYSCALL)
        "push rcx",             // User RIP (saved by SYSCALL)

        // Save callee-saved registers the C ABI expects preserved
        "push rbx",
        "push rbp",
        "push r12",
        "push r13",
        "push r14",
        "push r15",

        // Call the Rust syscall handler.
        // Args already in the right registers for the C ABI:
        //   RDI = arg0, RSI = arg1, RDX = arg2, R10 = arg3, R8 = arg4, R9 = arg5
        //   RAX = syscall number
        // Move R10 to RCX (C ABI 4th arg) since SYSCALL clobbers RCX.
        "mov rcx, r10",
        // RAX = syscall number, pass as first arg by moving everything:
        // We rearrange: handler(syscall_nr, arg0, arg1, arg2, arg3, arg4, arg5)
        // But our Rust handler signature is: fn(nr, a0, a1, a2, a3, a4) -> u64
        "push r9",              // 7th arg on stack (if needed)
        "push r8",              // Save
        "mov r9, r8",           // arg5 = r8
        "mov r8, rcx",          // arg4 = r10 (was moved to rcx)
        "mov rcx, rdx",        // arg3 = rdx
        "mov rdx, rsi",        // arg2 = rsi
        "mov rsi, rdi",        // arg1 = rdi
        "mov rdi, rax",        // arg0 = syscall number
        "pop r8",               // Restore r8 for arg5
        "add rsp, 8",          // Remove pushed r9

        "call {handler}",

        // RAX now has the return value

        // Restore callee-saved registers
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbp",
        "pop rbx",

        // Restore user RIP, RFLAGS, RSP
        "pop rcx",             // User RIP -> RCX for SYSRETQ
        "pop r11",             // User RFLAGS -> R11 for SYSRETQ
        "pop rsp",             // User RSP

        // Return to userspace
        "sysretq",

        kernel_stack_top = sym KERNEL_STACK_TOP,
        handler = sym super::handlers::syscall_dispatch,
    );
}

/// The kernel stack top pointer, set during init.
/// SYSCALL doesn't switch stacks, so we need a known kernel stack address.
#[used]
static mut KERNEL_STACK_TOP: u64 = 0;

/// Set the kernel stack pointer for SYSCALL entry.
pub fn set_kernel_stack(stack_top: u64) {
    unsafe {
        KERNEL_STACK_TOP = stack_top;
    }
}
