/// UDP header structure.
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct UdpHeader {
    pub src_port: u16,
    pub dst_port: u16,
    pub length: u16,
    pub checksum: u16,
}

impl UdpHeader {
    pub fn new(src_port: u16, dst_port: u16, payload_len: u16) -> Self {
        Self {
            src_port: src_port.to_be(),
            dst_port: dst_port.to_be(),
            length: (8 + payload_len).to_be(),
            checksum: 0,
        }
    }
}

/// Well-known UDP ports.
pub const PORT_DNS: u16 = 53;
pub const PORT_DHCP_CLIENT: u16 = 68;
pub const PORT_DHCP_SERVER: u16 = 67;
