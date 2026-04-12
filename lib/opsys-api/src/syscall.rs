/// Syscall numbers for the OpSys microkernel.
/// These are shared between the kernel and userspace.

pub const SYS_SEND: usize = 0;
pub const SYS_RECV: usize = 1;
pub const SYS_CALL: usize = 2;
pub const SYS_REPLY: usize = 3;
pub const SYS_NOTIFY: usize = 4;
pub const SYS_WAIT: usize = 5;
pub const SYS_YIELD: usize = 6;
pub const SYS_THREAD_CREATE: usize = 7;
pub const SYS_THREAD_RESUME: usize = 8;
pub const SYS_THREAD_SUSPEND: usize = 9;
pub const SYS_MAP: usize = 10;
pub const SYS_UNMAP: usize = 11;
pub const SYS_CAP_COPY: usize = 12;
pub const SYS_CAP_DELETE: usize = 13;
pub const SYS_CAP_REVOKE: usize = 14;
pub const SYS_DEBUG_PRINT: usize = 15;
pub const SYS_EXIT: usize = 16;
pub const SYS_IRQ_ACK: usize = 17;
