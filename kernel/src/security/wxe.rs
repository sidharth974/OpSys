use core::sync::atomic::{AtomicBool, Ordering};

/// Whether W^X enforcement is active.
static WXE_ENABLED: AtomicBool = AtomicBool::new(false);

/// Enable W^X (Write XOR Execute) enforcement.
/// Once enabled, any attempt to create a page mapping that is both
/// writable and executable will be rejected.
pub fn enable() {
    WXE_ENABLED.store(true, Ordering::SeqCst);
}

/// Check if W^X is currently enabled.
pub fn is_enabled() -> bool {
    WXE_ENABLED.load(Ordering::SeqCst)
}

/// Page protection flags for W^X validation.
#[derive(Debug, Clone, Copy)]
pub struct PageFlags {
    pub readable: bool,
    pub writable: bool,
    pub executable: bool,
    pub user: bool,
}

/// Validate page flags against W^X policy.
/// Returns Ok(()) if the flags are valid, Err with reason if not.
pub fn validate_flags(flags: PageFlags) -> Result<(), &'static str> {
    if !is_enabled() {
        return Ok(());
    }

    if flags.writable && flags.executable {
        return Err("W^X violation: page cannot be both writable and executable");
    }

    Ok(())
}

/// Check if a mapping request violates W^X.
/// For use in sys_map and the page fault handler.
pub fn check_mapping(writable: bool, executable: bool) -> bool {
    if !is_enabled() {
        return true; // allow
    }
    !(writable && executable)
}
