use x86_64::instructions::port::Port;
use alloc::vec::Vec;
use spin::Mutex;
use crate::serial_println;

/// Virtio device types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VirtioDeviceType {
    Network,
    Block,
    Console,
    Entropy,
    GPU,
    Input,
    Unknown(u16),
}

/// A detected virtio device.
#[derive(Debug, Clone)]
pub struct VirtioDevice {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub device_type: VirtioDeviceType,
    pub vendor_id: u16,
    pub device_id: u16,
    pub bar0: u32,
    pub irq: u8,
}

/// Global list of detected virtio devices.
pub static VIRTIO_DEVICES: Mutex<Vec<VirtioDevice>> = Mutex::new(Vec::new());

/// Detect all virtio devices from the PCI device list.
pub fn detect() {
    let pci_devices = crate::drivers::pci::PCI_DEVICES.lock();
    let mut virtio_devs = Vec::new();

    for dev in pci_devices.iter() {
        // Virtio vendor ID: 0x1AF4
        // Transitional device IDs: 0x1000-0x103F
        // Modern device IDs: 0x1040+
        if dev.vendor_id != 0x1AF4 {
            continue;
        }

        let dev_type = match dev.device_id {
            0x1000 | 0x1041 => VirtioDeviceType::Network,
            0x1001 | 0x1042 => VirtioDeviceType::Block,
            0x1003 | 0x1043 => VirtioDeviceType::Console,
            0x1005 | 0x1044 => VirtioDeviceType::Entropy,
            0x1050 => VirtioDeviceType::GPU,
            0x1052 => VirtioDeviceType::Input,
            id => VirtioDeviceType::Unknown(id),
        };

        serial_println!("[virtio] Found {:?} at {:02x}:{:02x}.{}",
            dev_type, dev.bus, dev.device, dev.function);

        // Read IRQ line
        let irq = crate::drivers::pci::pci_config_read_byte(
            dev.bus, dev.device, dev.function, 0x3C);

        virtio_devs.push(VirtioDevice {
            bus: dev.bus,
            device: dev.device,
            function: dev.function,
            device_type: dev_type,
            vendor_id: dev.vendor_id,
            device_id: dev.device_id,
            bar0: dev.bars[0],
            irq,
        });
    }

    serial_println!("[virtio] {} device(s) detected", virtio_devs.len());
    *VIRTIO_DEVICES.lock() = virtio_devs;
}
