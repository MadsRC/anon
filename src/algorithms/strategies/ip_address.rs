use crate::algorithms::entity_anonymization::PseudonymPools;
use crate::{AnonError, Result};
use rand::Rng;
use rand::rngs::StdRng;
use std::net::{Ipv4Addr, Ipv6Addr};

pub struct IpAddressStrategy;

impl IpAddressStrategy {
    pub fn redact(text: &str) -> String {
        "*".repeat(text.len())
    }

    pub fn suppress() -> String {
        "[IP_ADDRESS]".to_string()
    }

    pub fn generalize() -> String {
        "IP_ADDRESS".to_string()
    }

    pub fn pseudonymize(text: &str, rng: &mut StdRng, _pools: &PseudonymPools) -> Result<String> {
        Self::generate_class_preserving_ip_pseudonym(text, rng)
    }

    fn generate_class_preserving_ip_pseudonym(
        original_ip: &str,
        rng: &mut StdRng,
    ) -> Result<String> {
        // Check if it's CIDR notation first
        if let Some(slash_pos) = original_ip.find('/') {
            let (ip_part, prefix_part) = original_ip.split_at(slash_pos);
            let prefix_str = &prefix_part[1..]; // Skip the '/' character

            // Validate and preserve prefix length
            let prefix_len = prefix_str.parse::<u8>().map_err(|_| {
                AnonError::InvalidInput(format!("Invalid CIDR prefix length: {}", prefix_str))
            })?;

            // Generate pseudonym for the IP part
            let pseudo_ip = if let Ok(ipv4) = ip_part.parse::<Ipv4Addr>() {
                // Validate IPv4 prefix length
                if prefix_len > 32 {
                    return Err(AnonError::InvalidInput(format!(
                        "Invalid IPv4 prefix length: /{}",
                        prefix_len
                    )));
                }
                Self::generate_ipv4_pseudonym(ipv4, rng)
            } else if let Ok(ipv6) = ip_part.parse::<Ipv6Addr>() {
                // Validate IPv6 prefix length
                if prefix_len > 128 {
                    return Err(AnonError::InvalidInput(format!(
                        "Invalid IPv6 prefix length: /{}",
                        prefix_len
                    )));
                }
                Self::generate_ipv6_pseudonym(ipv6, rng)
            } else {
                return Err(AnonError::InvalidInput(format!(
                    "Invalid IP address in CIDR: {}",
                    ip_part
                )));
            };

            // Combine pseudonym IP with original prefix
            return Ok(format!("{}/{}", pseudo_ip, prefix_len));
        }

        // Not CIDR, handle as regular IP
        // Try parsing as IPv4 first
        if let Ok(ipv4) = original_ip.parse::<Ipv4Addr>() {
            return Ok(Self::generate_ipv4_pseudonym(ipv4, rng));
        }

        // Try parsing as IPv6
        if let Ok(ipv6) = original_ip.parse::<Ipv6Addr>() {
            return Ok(Self::generate_ipv6_pseudonym(ipv6, rng));
        }

        // Fallback for invalid IP addresses
        Err(AnonError::InvalidInput(format!(
            "Invalid IP address format: {}",
            original_ip
        )))
    }

    fn generate_ipv4_pseudonym(original: Ipv4Addr, rng: &mut StdRng) -> String {
        let octets = original.octets();

        // Classify the IPv4 address and generate pseudonym preserving the class
        if original.is_loopback() {
            // Loopback: 127.x.x.x
            format!(
                "127.{}.{}.{}",
                rng.gen_range(0..=255),
                rng.gen_range(0..=255),
                rng.gen_range(1..=255)
            )
        } else if Self::is_ipv4_private(octets) {
            // Private address - preserve private class
            let class = rng.gen_range(0..3);
            match class {
                0 => format!(
                    "192.168.{}.{}",
                    rng.gen_range(0..=255),
                    rng.gen_range(1..=254)
                ),
                1 => format!(
                    "10.{}.{}.{}",
                    rng.gen_range(0..=255),
                    rng.gen_range(0..=255),
                    rng.gen_range(1..=254)
                ),
                _ => format!(
                    "172.{}.{}.{}",
                    rng.gen_range(16..=31),
                    rng.gen_range(0..=255),
                    rng.gen_range(1..=254)
                ),
            }
        } else {
            // Public address - generate public address
            loop {
                let a = rng.gen_range(1..=223); // Avoid Class D (224-239) and Class E (240-255)
                let b = rng.gen_range(0..=255);
                let c = rng.gen_range(0..=255);
                let d = rng.gen_range(1..=254);

                let candidate = [a as u8, b as u8, c as u8, d as u8];

                // Ensure it's not private or loopback
                if !Self::is_ipv4_private(candidate) && a != 127 {
                    return format!("{}.{}.{}.{}", a, b, c, d);
                }
            }
        }
    }

    fn generate_ipv6_pseudonym(original: Ipv6Addr, rng: &mut StdRng) -> String {
        let segments = original.segments();

        if original.is_loopback() {
            // IPv6 loopback should remain ::1
            "::1".to_string()
        } else if original.is_unspecified() {
            // IPv6 unspecified should remain ::
            "::".to_string()
        } else if Self::is_ipv6_link_local(segments) {
            // Link-local: fe80::/10
            format!(
                "fe80::{:x}:{:x}:{:x}:{:x}",
                rng.gen_range(0..=0xffff),
                rng.gen_range(0..=0xffff),
                rng.gen_range(0..=0xffff),
                rng.gen_range(1..=0xffff)
            )
        } else if Self::is_ipv6_documentation(segments) {
            // Documentation prefix: 2001:db8::/32
            format!(
                "2001:db8::{:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
                rng.gen_range(0..=0xffff),
                rng.gen_range(0..=0xffff),
                rng.gen_range(0..=0xffff),
                rng.gen_range(0..=0xffff),
                rng.gen_range(0..=0xffff),
                rng.gen_range(1..=0xffff)
            )
        } else if Self::is_ipv4_mapped_ipv6(segments) {
            // IPv4-mapped IPv6: ::ffff:x.x.x.x
            let ipv4_part = format!(
                "{}.{}.{}.{}",
                rng.gen_range(1..=254),
                rng.gen_range(0..=255),
                rng.gen_range(0..=255),
                rng.gen_range(1..=254)
            );
            format!("::ffff:{}", ipv4_part)
        } else {
            // Global unicast - generate random global unicast address
            format!(
                "{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
                rng.gen_range(0x2000..=0x3fff), // Global unicast range
                rng.gen_range(0..=0xffff),
                rng.gen_range(0..=0xffff),
                rng.gen_range(0..=0xffff),
                rng.gen_range(0..=0xffff),
                rng.gen_range(0..=0xffff),
                rng.gen_range(0..=0xffff),
                rng.gen_range(1..=0xffff)
            )
        }
    }

    fn is_ipv4_private(octets: [u8; 4]) -> bool {
        match octets[0] {
            10 => true,                                    // 10.0.0.0/8
            172 if (16..=31).contains(&octets[1]) => true, // 172.16.0.0/12
            192 if octets[1] == 168 => true,               // 192.168.0.0/16
            _ => false,
        }
    }

    fn is_ipv6_link_local(segments: [u16; 8]) -> bool {
        (segments[0] & 0xffc0) == 0xfe80 // fe80::/10
    }

    fn is_ipv6_documentation(segments: [u16; 8]) -> bool {
        segments[0] == 0x2001 && segments[1] == 0x0db8 // 2001:db8::/32
    }

    fn is_ipv4_mapped_ipv6(segments: [u16; 8]) -> bool {
        segments[0..5] == [0, 0, 0, 0, 0] && segments[5] == 0xffff // ::ffff:0:0/96
    }
}
