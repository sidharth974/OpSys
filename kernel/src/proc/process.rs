use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use crate::cap::cspace::CSpace;

/// Process ID counter.
static NEXT_PID: AtomicUsize = AtomicUsize::new(1);

/// Process states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Active,
    Zombie,
}

/// A process: an address space + capability space + thread list.
/// For Phase 2, all threads share the kernel address space.
/// Per-process page tables will be added in Phase 3 (userspace).
pub struct Process {
    pub pid: usize,
    pub state: ProcessState,
    pub cspace: CSpace,
    pub thread_ids: Vec<usize>,
    pub name: &'static str,
}

impl Process {
    pub fn new(name: &'static str) -> Self {
        let pid = NEXT_PID.fetch_add(1, Ordering::Relaxed);
        Self {
            pid,
            state: ProcessState::Active,
            cspace: CSpace::new(),
            thread_ids: Vec::new(),
            name,
        }
    }
}
