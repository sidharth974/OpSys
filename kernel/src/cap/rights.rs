use bitflags::bitflags;

bitflags! {
    /// Access rights attached to a capability.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Rights: u32 {
        const READ    = 1 << 0;
        const WRITE   = 1 << 1;
        const EXECUTE = 1 << 2;
        /// Can delegate (copy) this capability to another process.
        const GRANT   = 1 << 3;
        /// Can revoke all capabilities derived from this one.
        const REVOKE  = 1 << 4;

        const ALL = Self::READ.bits() | Self::WRITE.bits() | Self::EXECUTE.bits()
                  | Self::GRANT.bits() | Self::REVOKE.bits();
    }
}
