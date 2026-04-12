pub mod wxe;
pub mod cpu_protection;

use alloc::vec::Vec;
use alloc::string::String;
use crate::serial_println;

/// Security feature status for reporting.
#[derive(Debug)]
pub struct SecurityStatus {
    pub features: Vec<(String, bool, String)>, // (name, enabled, description)
}

impl SecurityStatus {
    pub fn collect() -> Self {
        let mut features = Vec::new();

        // W^X enforcement
        features.push((
            String::from("W^X"),
            wxe::is_enabled(),
            String::from("Write XOR Execute - pages cannot be both writable and executable"),
        ));

        // SMEP
        let smep = cpu_protection::smep_enabled();
        features.push((
            String::from("SMEP"),
            smep,
            String::from("Supervisor Mode Execution Prevention - kernel cannot execute user pages"),
        ));

        // SMAP
        let smap = cpu_protection::smap_enabled();
        features.push((
            String::from("SMAP"),
            smap,
            String::from("Supervisor Mode Access Prevention - kernel cannot read/write user pages"),
        ));

        // NX bit
        let nx = cpu_protection::nx_enabled();
        features.push((
            String::from("NX"),
            nx,
            String::from("No-Execute bit - data pages cannot be executed"),
        ));

        // Capability-based security
        features.push((
            String::from("Capabilities"),
            true,
            String::from("Unforgeable tokens for all resource access (seL4-style)"),
        ));

        // Microkernel isolation
        features.push((
            String::from("Microkernel"),
            true,
            String::from("Drivers in userspace - driver crash cannot take down kernel"),
        ));

        Self { features }
    }

    pub fn print_report(&self) {
        serial_println!("Security Status:");
        serial_println!("  {:<16} {:<8} {}", "FEATURE", "STATUS", "DESCRIPTION");
        serial_println!("  {:<16} {:<8} {}", "-------", "------", "-----------");
        for (name, enabled, desc) in &self.features {
            let status = if *enabled { "ON" } else { "OFF" };
            serial_println!("  {:<16} {:<8} {}", name, status, desc);
        }
    }
}

/// Initialize all security features.
pub fn init() {
    serial_println!("[security] Initializing security features...");

    // Enable W^X policy
    wxe::enable();
    serial_println!("[security] W^X enforcement: ENABLED");

    // Enable SMEP if supported
    if cpu_protection::try_enable_smep() {
        serial_println!("[security] SMEP: ENABLED");
    } else {
        serial_println!("[security] SMEP: not supported by CPU");
    }

    // Enable SMAP if supported
    if cpu_protection::try_enable_smap() {
        serial_println!("[security] SMAP: ENABLED");
    } else {
        serial_println!("[security] SMAP: not supported by CPU");
    }

    // Check NX bit
    if cpu_protection::nx_enabled() {
        serial_println!("[security] NX bit: ENABLED");
    } else {
        serial_println!("[security] NX bit: not available");
    }
}
