use super::rights::Rights;
use super::object::KernelObject;

/// A capability: an unforgeable token granting access to a kernel object.
/// Capabilities live only in kernel memory and are addressed by slot index.
#[derive(Debug, Clone)]
pub struct Capability {
    /// The kernel object this capability grants access to.
    pub object: KernelObject,
    /// The access rights granted by this capability.
    pub rights: Rights,
    /// Badge: an opaque value set by the creator, used by servers to
    /// identify which client sent a message.
    pub badge: u64,
}

impl Capability {
    pub fn new(object: KernelObject, rights: Rights) -> Self {
        Self {
            object,
            rights,
            badge: 0,
        }
    }

    pub fn with_badge(mut self, badge: u64) -> Self {
        self.badge = badge;
        self
    }

    /// Check if this capability has the required rights.
    pub fn has_rights(&self, required: Rights) -> bool {
        self.rights.contains(required)
    }

    /// Create a derived capability with reduced rights.
    /// Returns None if the requested rights exceed the current rights.
    pub fn derive(&self, new_rights: Rights) -> Option<Self> {
        if self.rights.contains(new_rights) {
            Some(Self {
                object: self.object,
                rights: new_rights,
                badge: self.badge,
            })
        } else {
            None
        }
    }
}
