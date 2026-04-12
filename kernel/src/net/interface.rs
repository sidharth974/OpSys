use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;

/// Global network interface list.
pub static INTERFACES: Mutex<Vec<NetInterface>> = Mutex::new(Vec::new());

/// A network interface.
#[derive(Debug, Clone)]
pub struct NetInterface {
    pub name: String,
    pub mac: [u8; 6],
    pub ip: [u8; 4],
    pub netmask: [u8; 4],
    pub gateway: [u8; 4],
    pub link_up: bool,
    pub vendor_id: u16,
    pub device_id: u16,
    pub rx_packets: u64,
    pub tx_packets: u64,
}

impl NetInterface {
    pub fn mac_str(&self) -> String {
        alloc::format!("{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.mac[0], self.mac[1], self.mac[2],
            self.mac[3], self.mac[4], self.mac[5])
    }

    pub fn ip_str(&self) -> String {
        alloc::format!("{}.{}.{}.{}", self.ip[0], self.ip[1], self.ip[2], self.ip[3])
    }
}

/// Detect network interfaces from PCI devices.
pub fn detect() {
    let pci_devices = crate::drivers::pci::PCI_DEVICES.lock();
    let mut interfaces = Vec::new();
    let mut idx = 0;

    for dev in pci_devices.iter() {
        // Class 0x02 = Network Controller
        if dev.class_code == 0x02 {
            let name = alloc::format!("eth{}", idx);
            // Generate a deterministic MAC from PCI address
            let mac = [0x52, 0x54, 0x00, dev.bus, dev.device, dev.function];
            interfaces.push(NetInterface {
                name,
                mac,
                ip: [10, 0, 2, 15],       // QEMU default user-mode IP
                netmask: [255, 255, 255, 0],
                gateway: [10, 0, 2, 2],    // QEMU default gateway
                link_up: true,
                vendor_id: dev.vendor_id,
                device_id: dev.device_id,
                rx_packets: 0,
                tx_packets: 0,
            });
            idx += 1;
        }
    }
    *INTERFACES.lock() = interfaces;
}
