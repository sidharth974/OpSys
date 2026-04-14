use alloc::vec::Vec;
use spin::Mutex;
use x86_64::instructions::port::Port;
use crate::serial_println;

/// PCI configuration space access ports.
const PCI_CONFIG_ADDR: u16 = 0xCF8;
const PCI_CONFIG_DATA: u16 = 0xCFC;

/// Global list of discovered PCI devices.
pub static PCI_DEVICES: Mutex<Vec<PciDevice>> = Mutex::new(Vec::new());

/// A discovered PCI device.
#[derive(Debug, Clone)]
pub struct PciDevice {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub revision: u8,
    pub header_type: u8,
    pub bars: [u32; 6],
}

impl PciDevice {
    /// Human-readable class name.
    pub fn class_name(&self) -> &'static str {
        match (self.class_code, self.subclass) {
            (0x00, _) => "Unclassified",
            (0x01, 0x00) => "SCSI Storage",
            (0x01, 0x01) => "IDE Controller",
            (0x01, 0x06) => "SATA Controller",
            (0x01, 0x08) => "NVMe Controller",
            (0x01, _) => "Mass Storage",
            (0x02, 0x00) => "Ethernet Controller",
            (0x02, _) => "Network Controller",
            (0x03, 0x00) => "VGA Controller",
            (0x03, _) => "Display Controller",
            (0x04, _) => "Multimedia",
            (0x05, _) => "Memory Controller",
            (0x06, 0x00) => "Host Bridge",
            (0x06, 0x01) => "ISA Bridge",
            (0x06, 0x04) => "PCI-PCI Bridge",
            (0x06, _) => "Bridge Device",
            (0x07, _) => "Serial Controller",
            (0x08, _) => "System Peripheral",
            (0x0C, 0x03) => "USB Controller",
            (0x0C, _) => "Serial Bus",
            (0x0D, _) => "Wireless Controller",
            _ => "Unknown",
        }
    }
}

/// Read a 32-bit value from PCI configuration space.
fn pci_config_read(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let address: u32 = (1 << 31) // Enable bit
        | ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((function as u32) << 8)
        | ((offset as u32) & 0xFC);

    unsafe {
        let mut addr_port = Port::<u32>::new(PCI_CONFIG_ADDR);
        let mut data_port = Port::<u32>::new(PCI_CONFIG_DATA);
        addr_port.write(address);
        data_port.read()
    }
}

/// Read a byte from PCI config space.
pub fn pci_config_read_byte(bus: u8, dev: u8, func: u8, offset: u8) -> u8 {
    let val = pci_config_read(bus, dev, func, offset & 0xFC);
    ((val >> ((offset & 3) * 8)) & 0xFF) as u8
}

/// Read vendor and device ID.
fn read_vendor_device(bus: u8, dev: u8, func: u8) -> (u16, u16) {
    let val = pci_config_read(bus, dev, func, 0x00);
    let vendor = val as u16;
    let device = (val >> 16) as u16;
    (vendor, device)
}

/// Enumerate all PCI devices on the bus.
pub fn enumerate() {
    let mut devices = Vec::new();

    for bus in 0..=255u8 {
        for device in 0..32u8 {
            let (vendor, _) = read_vendor_device(bus, device, 0);
            if vendor == 0xFFFF {
                continue;
            }

            // Check all 8 functions
            for function in 0..8u8 {
                let (vendor_id, device_id) = read_vendor_device(bus, device, function);
                if vendor_id == 0xFFFF {
                    continue;
                }

                let class_reg = pci_config_read(bus, device, function, 0x08);
                let revision = class_reg as u8;
                let prog_if = (class_reg >> 8) as u8;
                let subclass = (class_reg >> 16) as u8;
                let class_code = (class_reg >> 24) as u8;

                let header_reg = pci_config_read(bus, device, function, 0x0C);
                let header_type = (header_reg >> 16) as u8 & 0x7F;

                // Read BARs (only for type 0 headers)
                let mut bars = [0u32; 6];
                if header_type == 0 {
                    for i in 0..6 {
                        bars[i] = pci_config_read(bus, device, function, 0x10 + (i as u8) * 4);
                    }
                }

                devices.push(PciDevice {
                    bus,
                    device,
                    function,
                    vendor_id,
                    device_id,
                    class_code,
                    subclass,
                    prog_if,
                    revision,
                    header_type,
                    bars,
                });

                // If not multi-function, stop after function 0
                if function == 0 {
                    let full_header = (pci_config_read(bus, device, 0, 0x0C) >> 16) as u8;
                    if full_header & 0x80 == 0 {
                        break;
                    }
                }
            }
        }
    }

    serial_println!("[pci] Found {} devices", devices.len());
    for dev in &devices {
        serial_println!(
            "[pci]   {:02x}:{:02x}.{} {:04x}:{:04x} {}",
            dev.bus, dev.device, dev.function,
            dev.vendor_id, dev.device_id,
            dev.class_name()
        );
    }

    *PCI_DEVICES.lock() = devices;
}
