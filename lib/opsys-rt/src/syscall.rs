use opsys_api::syscall::*;

/// Raw syscall: execute the SYSCALL instruction.
/// nr = syscall number, a0-a4 = arguments.
/// Returns the value in RAX.
#[inline(always)]
pub fn raw_syscall(nr: usize, a0: usize, a1: usize, a2: usize, a3: usize, a4: usize) -> isize {
    let ret: isize;
    unsafe {
        core::arch::asm!(
            "syscall",
            inlateout("rax") nr as u64 => ret,
            in("rdi") a0 as u64,
            in("rsi") a1 as u64,
            in("rdx") a2 as u64,
            in("r10") a3 as u64,
            in("r8") a4 as u64,
            // SYSCALL clobbers RCX and R11
            out("rcx") _,
            out("r11") _,
        );
    }
    ret
}

pub fn debug_print(s: &str) {
    raw_syscall(SYS_DEBUG_PRINT, s.as_ptr() as usize, s.len(), 0, 0, 0);
}

pub fn exit(code: usize) -> ! {
    raw_syscall(SYS_EXIT, code, 0, 0, 0, 0);
    loop {}
}

pub fn yield_now() {
    raw_syscall(SYS_YIELD, 0, 0, 0, 0, 0);
}
