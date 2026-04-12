use super::vfs::VFS;

/// Populate /dev with device nodes.
pub fn init() {
    let mut vfs = VFS.lock();
    let dev = vfs.resolve_path("/dev").unwrap();
    vfs.create_file(dev, "null", b"");
    vfs.create_file(dev, "zero", b"");
    vfs.create_file(dev, "random", b"OpSys PRNG not yet implemented");
    vfs.create_file(dev, "console", b"");
    vfs.create_file(dev, "serial0", b"COM1 @ 0x3F8");

    // List PCI devices
    let devices = crate::drivers::pci::PCI_DEVICES.lock();
    for dev_info in devices.iter() {
        let name = alloc::format!("pci_{:02x}{:02x}{}",
            dev_info.bus, dev_info.device, dev_info.function);
        let data = alloc::format!("{:04x}:{:04x} {}\n",
            dev_info.vendor_id, dev_info.device_id, dev_info.class_name());
        let dev_dir = vfs.resolve_path("/dev").unwrap();
        vfs.create_file(dev_dir, &name, data.as_bytes());
    }
}
