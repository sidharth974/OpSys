/// Kernel error codes shared between kernel and userspace.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(isize)]
pub enum Error {
    /// No error
    Success = 0,
    /// Invalid capability slot
    InvalidCap = -1,
    /// Insufficient rights on capability
    InsufficientRights = -2,
    /// Invalid syscall number
    InvalidSyscall = -3,
    /// Out of memory
    OutOfMemory = -4,
    /// Invalid argument
    InvalidArg = -5,
    /// Resource busy
    Busy = -6,
    /// Operation would block
    WouldBlock = -7,
    /// Object not found
    NotFound = -8,
    /// Permission denied
    PermissionDenied = -9,
}
