/// Syscall numbers for the OpSys microkernel.

// IPC
pub const SYS_SEND: usize = 0;
pub const SYS_RECV: usize = 1;
pub const SYS_CALL: usize = 2;
pub const SYS_REPLY: usize = 3;
pub const SYS_NOTIFY: usize = 4;
pub const SYS_WAIT: usize = 5;

// Scheduling
pub const SYS_YIELD: usize = 6;
pub const SYS_EXIT: usize = 16;

// Thread management
pub const SYS_THREAD_CREATE: usize = 7;
pub const SYS_THREAD_RESUME: usize = 8;
pub const SYS_THREAD_SUSPEND: usize = 9;

// Memory
pub const SYS_MAP: usize = 10;
pub const SYS_UNMAP: usize = 11;
pub const SYS_MMAP: usize = 30;

// Capabilities
pub const SYS_CAP_COPY: usize = 12;
pub const SYS_CAP_DELETE: usize = 13;
pub const SYS_CAP_REVOKE: usize = 14;

// Debug
pub const SYS_DEBUG_PRINT: usize = 15;

// IRQ
pub const SYS_IRQ_ACK: usize = 17;

// File operations (POSIX-like)
pub const SYS_OPEN: usize = 20;
pub const SYS_CLOSE: usize = 21;
pub const SYS_READ: usize = 22;
pub const SYS_WRITE: usize = 23;
pub const SYS_STAT: usize = 24;
pub const SYS_READDIR: usize = 25;
pub const SYS_MKDIR: usize = 26;
pub const SYS_UNLINK: usize = 27;

// Process management
pub const SYS_FORK: usize = 40;
pub const SYS_EXEC: usize = 41;
pub const SYS_WAITPID: usize = 42;
pub const SYS_GETPID: usize = 43;

// Network
pub const SYS_SOCKET: usize = 50;
pub const SYS_BIND: usize = 51;
pub const SYS_SENDTO: usize = 52;
pub const SYS_RECVFROM: usize = 53;

// Time
pub const SYS_CLOCK_GETTIME: usize = 60;
pub const SYS_NANOSLEEP: usize = 61;

// System info
pub const SYS_UNAME: usize = 70;
pub const SYS_SYSINFO: usize = 71;
