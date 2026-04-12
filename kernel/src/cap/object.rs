/// Types of kernel objects that capabilities can reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    /// IPC endpoint for synchronous message passing.
    Endpoint,
    /// Async notification object (bitmask signal).
    Notification,
    /// A region of physical memory.
    MemoryRegion,
    /// A thread control block.
    Thread,
    /// A process (address space + capability space).
    Process,
    /// An IRQ handler binding.
    IrqHandler,
    /// An I/O port range.
    IoPort,
    /// Device MMIO memory region.
    DeviceMemory,
}

/// A kernel object referenced by capabilities.
/// Each variant holds an index/ID into the relevant kernel table.
#[derive(Debug, Clone, Copy)]
pub enum KernelObject {
    Endpoint(usize),
    Notification(usize),
    MemoryRegion { base: u64, size: u64 },
    Thread(usize),
    Process(usize),
    IrqHandler(u8),
    IoPort { base: u16, count: u16 },
    DeviceMemory { phys_base: u64, size: u64 },
}

impl KernelObject {
    pub fn object_type(&self) -> ObjectType {
        match self {
            KernelObject::Endpoint(_) => ObjectType::Endpoint,
            KernelObject::Notification(_) => ObjectType::Notification,
            KernelObject::MemoryRegion { .. } => ObjectType::MemoryRegion,
            KernelObject::Thread(_) => ObjectType::Thread,
            KernelObject::Process(_) => ObjectType::Process,
            KernelObject::IrqHandler(_) => ObjectType::IrqHandler,
            KernelObject::IoPort { .. } => ObjectType::IoPort,
            KernelObject::DeviceMemory { .. } => ObjectType::DeviceMemory,
        }
    }
}
