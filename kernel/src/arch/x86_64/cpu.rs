use core::arch::x86_64::__cpuid;
use crate::serial_println;

/// Detect and report CPU features relevant to the OS.
pub fn detect_features() {
    let vendor = get_vendor_string();
    serial_println!("[cpu] Vendor: {}", core::str::from_utf8(&vendor).unwrap_or("unknown"));

    let cpuid1 = __cpuid(1);
    let has_sse = cpuid1.edx & (1 << 25) != 0;
    let has_sse2 = cpuid1.edx & (1 << 26) != 0;
    let has_avx = cpuid1.ecx & (1 << 28) != 0;

    serial_println!("[cpu] SSE: {}, SSE2: {}, AVX: {}", has_sse, has_sse2, has_avx);

    // Check extended features for AVX2 and AVX-512
    let cpuid7 = __cpuid(7);
    let has_avx2 = cpuid7.ebx & (1 << 5) != 0;
    let has_avx512f = cpuid7.ebx & (1 << 16) != 0;

    serial_println!("[cpu] AVX2: {}, AVX-512F: {}", has_avx2, has_avx512f);
}

fn get_vendor_string() -> [u8; 12] {
    let cpuid0 = __cpuid(0);
    let mut vendor = [0u8; 12];
    vendor[0..4].copy_from_slice(&cpuid0.ebx.to_le_bytes());
    vendor[4..8].copy_from_slice(&cpuid0.edx.to_le_bytes());
    vendor[8..12].copy_from_slice(&cpuid0.ecx.to_le_bytes());
    vendor
}
