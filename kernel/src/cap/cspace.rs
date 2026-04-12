use alloc::vec::Vec;
use super::capability::Capability;

/// Maximum number of capability slots per process.
pub const MAX_CAPS: usize = 256;

/// Capability Space: per-process table of capabilities.
/// Syscalls reference kernel objects by slot index into this table.
pub struct CSpace {
    slots: Vec<Option<Capability>>,
}

impl CSpace {
    pub fn new() -> Self {
        let mut slots = Vec::with_capacity(MAX_CAPS);
        slots.resize_with(MAX_CAPS, || None);
        Self { slots }
    }

    /// Insert a capability into the first free slot. Returns the slot index.
    pub fn insert(&mut self, cap: Capability) -> Option<usize> {
        for (i, slot) in self.slots.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(cap);
                return Some(i);
            }
        }
        None
    }

    /// Insert a capability at a specific slot.
    pub fn insert_at(&mut self, index: usize, cap: Capability) -> bool {
        if index >= self.slots.len() {
            return false;
        }
        self.slots[index] = Some(cap);
        true
    }

    /// Look up a capability by slot index.
    pub fn get(&self, index: usize) -> Option<&Capability> {
        self.slots.get(index)?.as_ref()
    }

    /// Remove a capability from a slot.
    pub fn remove(&mut self, index: usize) -> Option<Capability> {
        if index >= self.slots.len() {
            return None;
        }
        self.slots[index].take()
    }
}
