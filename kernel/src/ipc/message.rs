/// IPC message: small enough to pass entirely in registers (64 bytes).
/// This is the fundamental communication unit in the microkernel.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Message {
    /// Message label: identifies the operation/type.
    pub label: u64,
    /// Payload: up to 7 register-sized words.
    pub words: [u64; 7],
}

impl Message {
    pub const fn empty() -> Self {
        Self {
            label: 0,
            words: [0; 7],
        }
    }

    pub const fn new(label: u64) -> Self {
        Self {
            label,
            words: [0; 7],
        }
    }
}
