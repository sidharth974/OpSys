use alloc::string::String;

/// DNS record types.
pub const DNS_TYPE_A: u16 = 1;     // IPv4 address
pub const DNS_TYPE_AAAA: u16 = 28; // IPv6 address
pub const DNS_TYPE_CNAME: u16 = 5; // Canonical name
pub const DNS_TYPE_MX: u16 = 15;   // Mail exchange

/// DNS header (12 bytes).
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct DnsHeader {
    pub id: u16,
    pub flags: u16,
    pub qcount: u16,
    pub acount: u16,
    pub nscount: u16,
    pub arcount: u16,
}

/// Build a DNS query packet for a hostname.
pub fn build_query(hostname: &str, query_type: u16) -> alloc::vec::Vec<u8> {
    let mut packet = alloc::vec::Vec::new();

    // Header
    packet.extend_from_slice(&42u16.to_be_bytes());   // ID
    packet.extend_from_slice(&0x0100u16.to_be_bytes()); // Flags: standard query, recursion desired
    packet.extend_from_slice(&1u16.to_be_bytes());     // 1 question
    packet.extend_from_slice(&0u16.to_be_bytes());     // 0 answers
    packet.extend_from_slice(&0u16.to_be_bytes());     // 0 authority
    packet.extend_from_slice(&0u16.to_be_bytes());     // 0 additional

    // Question: encode hostname as DNS labels
    for label in hostname.split('.') {
        packet.push(label.len() as u8);
        packet.extend_from_slice(label.as_bytes());
    }
    packet.push(0); // Root label

    packet.extend_from_slice(&query_type.to_be_bytes()); // Type
    packet.extend_from_slice(&1u16.to_be_bytes());       // Class IN

    packet
}
