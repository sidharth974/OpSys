use core::arch::x86_64::__cpuid;

/// Check if the CPU supports SMEP (Supervisor Mode Execution Prevention).
pub fn cpu_supports_smep() -> bool {
    let cpuid7 = __cpuid(7);
    cpuid7.ebx & (1 << 7) != 0
}

/// Check if the CPU supports SMAP (Supervisor Mode Access Prevention).
pub fn cpu_supports_smap() -> bool {
    let cpuid7 = __cpuid(7);
    cpuid7.ebx & (1 << 20) != 0
}

/// Check if SMEP is currently enabled in CR4.
pub fn smep_enabled() -> bool {
    let cr4: u64;
    unsafe {
        core::arch::asm!("mov {}, cr4", out(reg) cr4);
    }
    cr4 & (1 << 20) != 0
}

/// Check if SMAP is currently enabled in CR4.
pub fn smap_enabled() -> bool {
    let cr4: u64;
    unsafe {
        core::arch::asm!("mov {}, cr4", out(reg) cr4);
    }
    cr4 & (1 << 21) != 0
}

/// Try to enable SMEP. Returns true if successfully enabled.
pub fn try_enable_smep() -> bool {
    if !cpu_supports_smep() {
        return false;
    }
    unsafe {
        let mut cr4: u64;
        core::arch::asm!("mov {}, cr4", out(reg) cr4);
        cr4 |= 1 << 20; // SMEP bit
        core::arch::asm!("mov cr4, {}", in(reg) cr4);
    }
    true
}

/// Try to enable SMAP. Returns true if successfully enabled.
pub fn try_enable_smap() -> bool {
    if !cpu_supports_smap() {
        return false;
    }
    unsafe {
        let mut cr4: u64;
        core::arch::asm!("mov {}, cr4", out(reg) cr4);
        cr4 |= 1 << 21; // SMAP bit
        core::arch::asm!("mov cr4, {}", in(reg) cr4);
    }
    true
}

/// Check if NX (No-Execute) bit is enabled in the EFER MSR.
pub fn nx_enabled() -> bool {
    let efer: u64;
    unsafe {
        core::arch::asm!("rdmsr", in("ecx") 0xC0000080u32, out("eax") _, out("edx") _);
        // Read EFER properly
        let lo: u32;
        let hi: u32;
        core::arch::asm!("rdmsr", in("ecx") 0xC0000080u32, out("eax") lo, out("edx") hi);
        efer = (hi as u64) << 32 | lo as u64;
    }
    efer & (1 << 11) != 0 // NXE bit
}
