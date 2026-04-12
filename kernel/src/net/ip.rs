/// IPv4 header structure.
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Ipv4Header {
    pub version_ihl: u8,      // Version (4) + IHL (4)
    pub tos: u8,
    pub total_length: u16,
    pub identification: u16,
    pub flags_fragment: u16,
    pub ttl: u8,
    pub protocol: u8,          // 6=TCP, 17=UDP
    pub checksum: u16,
    pub src_ip: [u8; 4],
    pub dst_ip: [u8; 4],
}

impl Ipv4Header {
    pub fn new(src: [u8; 4], dst: [u8; 4], protocol: u8, payload_len: u16) -> Self {
        let total = 20 + payload_len;
        Self {
            version_ihl: 0x45,     // IPv4, 5 words (20 bytes)
            tos: 0,
            total_length: total.to_be(),
            identification: 0,
            flags_fragment: 0,
            ttl: 64,
            protocol,
            checksum: 0,           // Calculated later
            src_ip: src,
            dst_ip: dst,
        }
    }

    pub fn src_str(&self) -> alloc::string::String {
        alloc::format!("{}.{}.{}.{}", self.src_ip[0], self.src_ip[1], self.src_ip[2], self.src_ip[3])
    }

    pub fn dst_str(&self) -> alloc::string::String {
        alloc::format!("{}.{}.{}.{}", self.dst_ip[0], self.dst_ip[1], self.dst_ip[2], self.dst_ip[3])
    }
}

/// Protocol numbers.
pub const PROTO_ICMP: u8 = 1;
pub const PROTO_TCP: u8 = 6;
pub const PROTO_UDP: u8 = 17;
