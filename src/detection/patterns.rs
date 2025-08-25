use super::{DetectedEntity, EntityDetector, EntityType};
use crate::{AnonError, Result};
use regex::Regex;
use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};

pub struct PatternDetector {
    patterns: HashMap<EntityType, Regex>,
}

impl PatternDetector {
    pub fn new() -> Result<Self> {
        let mut patterns = HashMap::new();

        patterns.insert(
            EntityType::Email,
            Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b")
                .map_err(|e| AnonError::AlgorithmError(format!("Invalid regex: {}", e)))?,
        );

        patterns.insert(
            EntityType::PhoneNumber,
            Regex::new(r"(?:\+?1[-.\s]?)?\(?[0-9]{3}\)?[-.\s]?[0-9]{3}[-.\s]?[0-9]{4}")
                .map_err(|e| AnonError::AlgorithmError(format!("Invalid regex: {}", e)))?,
        );

        patterns.insert(
            EntityType::SocialSecurityNumber,
            Regex::new(r"\b\d{3}-?\d{2}-?\d{4}\b")
                .map_err(|e| AnonError::AlgorithmError(format!("Invalid regex: {}", e)))?,
        );

        patterns.insert(
            EntityType::CreditCard,
            Regex::new(r"\b(?:4[0-9]{3}[-\s]?[0-9]{4}[-\s]?[0-9]{4}[-\s]?[0-9]{4}|5[1-5][0-9]{2}[-\s]?[0-9]{4}[-\s]?[0-9]{4}[-\s]?[0-9]{4}|3[47][0-9]{1,2}[-\s]?[0-9]{6}[-\s]?[0-9]{5})\b")
                .map_err(|e| AnonError::AlgorithmError(format!("Invalid regex: {}", e)))?,
        );

        // We'll handle IP addresses separately using a word-based approach
        // Remove the IP pattern from regex patterns since we handle it specially

        Ok(Self { patterns })
    }

    pub fn add_pattern(&mut self, entity_type: EntityType, pattern: &str) -> Result<()> {
        let regex = Regex::new(pattern)
            .map_err(|e| AnonError::AlgorithmError(format!("Invalid regex: {}", e)))?;
        self.patterns.insert(entity_type, regex);
        Ok(())
    }

    fn is_valid_ip_address(&self, text: &str) -> bool {
        // Check for CIDR notation first
        if let Some(slash_pos) = text.find('/') {
            let (ip_part, prefix_part) = text.split_at(slash_pos);
            let prefix_str = &prefix_part[1..]; // Skip the '/' character

            // Validate prefix length
            if let Ok(prefix_len) = prefix_str.parse::<u8>() {
                // Check if it's IPv4 CIDR
                if ip_part.parse::<Ipv4Addr>().is_ok() {
                    return prefix_len <= 32;
                }

                // Check if it's IPv6 CIDR
                if ip_part.parse::<Ipv6Addr>().is_ok() {
                    return prefix_len <= 128;
                }
            }

            // Invalid CIDR format
            return false;
        }

        // Not CIDR, try parsing as regular IP
        // Try parsing as IPv4 first
        if text.parse::<Ipv4Addr>().is_ok() {
            return true;
        }

        // Try parsing as IPv6
        if text.parse::<Ipv6Addr>().is_ok() {
            return true;
        }

        false
    }

    fn detect_ip_addresses(&self, text: &str) -> Result<Vec<DetectedEntity>> {
        let mut entities = Vec::new();

        // Split text into words and potential IP candidates
        let mut current_pos = 0;
        for word in text.split_whitespace() {
            // Find the position of this word in the original text
            if let Some(word_start) = text[current_pos..].find(word) {
                let word_start = current_pos + word_start;
                let word_end = word_start + word.len();

                // Check if this word looks like an IP address candidate
                if self.looks_like_ip_candidate(word) {
                    // Try to parse it as a valid IP
                    if self.is_valid_ip_address(word) {
                        entities.push(DetectedEntity::new(
                            EntityType::IpAddress,
                            word.to_string(),
                            word_start,
                            word_end,
                            1.0,
                        ));
                    }
                }
                current_pos = word_end;
            }
        }

        // Also check for IPv6 addresses that might contain spaces or special cases
        self.detect_ipv6_special_cases(text, &mut entities)?;

        Ok(entities)
    }

    fn looks_like_ip_candidate(&self, word: &str) -> bool {
        // IPv4 or IPv4 CIDR: contains dots and digits (and possibly slash for CIDR)
        if word.contains('.') && word.chars().any(|c| c.is_ascii_digit()) {
            return true;
        }

        // IPv6 or IPv6 CIDR: contains colons and hex characters (and possibly slash for CIDR)
        if word.contains(':') && word.chars().any(|c| c.is_ascii_hexdigit()) {
            return true;
        }

        false
    }

    fn detect_ipv6_special_cases(
        &self,
        text: &str,
        entities: &mut Vec<DetectedEntity>,
    ) -> Result<()> {
        // Handle IPv6 addresses that might be split by whitespace in our word-based approach
        // Look for patterns like "2001:0db8:85a3:0000:0000:8a2e:0370:7334"

        // Updated IPv6 regex to include optional CIDR notation (/prefix)
        let ipv6_regex = Regex::new(r"(?:(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}|(?:[0-9a-fA-F]{1,4}:){1,7}:|(?:[0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|(?:[0-9a-fA-F]{1,4}:){1,5}(?::[0-9a-fA-F]{1,4}){1,2}|(?:[0-9a-fA-F]{1,4}:){1,4}(?::[0-9a-fA-F]{1,4}){1,3}|(?:[0-9a-fA-F]{1,4}:){1,3}(?::[0-9a-fA-F]{1,4}){1,4}|(?:[0-9a-fA-F]{1,4}:){1,2}(?::[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:(?::[0-9a-fA-F]{1,4}){1,6}|:(?::[0-9a-fA-F]{1,4}){1,7}|::1?|::|::ffff:(?:[0-9]{1,3}\.){3}[0-9]{1,3}|fe80:(?::[0-9a-fA-F]{1,4}){0,4})(?:/[0-9]{1,3})?")
            .map_err(|e| AnonError::AlgorithmError(format!("Invalid IPv6 regex: {}", e)))?;

        for mat in ipv6_regex.find_iter(text) {
            let candidate = mat.as_str();
            if self.is_valid_ip_address(candidate) {
                // Check if we already have this IP address detected
                let already_exists = entities.iter().any(|e| {
                    e.start == mat.start()
                        && e.end == mat.end()
                        && e.entity_type == EntityType::IpAddress
                });

                if !already_exists {
                    entities.push(DetectedEntity::new(
                        EntityType::IpAddress,
                        candidate.to_string(),
                        mat.start(),
                        mat.end(),
                        1.0,
                    ));
                }
            }
        }

        Ok(())
    }
}

impl EntityDetector for PatternDetector {
    fn detect(&mut self, text: &str) -> Result<Vec<DetectedEntity>> {
        let mut entities = Vec::new();

        // Handle regex-based patterns (excluding IP addresses)
        for (entity_type, regex) in &self.patterns {
            for mat in regex.find_iter(text) {
                entities.push(DetectedEntity::new(
                    entity_type.clone(),
                    mat.as_str().to_string(),
                    mat.start(),
                    mat.end(),
                    1.0, // Pattern matches have 100% confidence
                ));
            }
        }

        // Handle IP addresses separately using word-based detection
        entities.extend(self.detect_ip_addresses(text)?);

        // Sort by start position first, then by length (longest first) to handle overlapping matches
        entities.sort_by(|a, b| a.start.cmp(&b.start).then_with(|| b.end.cmp(&a.end)));

        // Remove overlapping entities, keeping the longest ones
        let mut filtered_entities = Vec::new();
        for entity in entities {
            let overlaps = filtered_entities.iter().any(|existing: &DetectedEntity| {
                existing.entity_type == entity.entity_type
                    && existing.start <= entity.start
                    && existing.end >= entity.end
            });

            if !overlaps {
                filtered_entities.push(entity);
            }
        }

        filtered_entities.sort_by_key(|e| e.start);
        Ok(filtered_entities)
    }

    fn supported_entities(&self) -> Vec<EntityType> {
        let mut entities: Vec<EntityType> = self.patterns.keys().cloned().collect();
        // Always include IP address since we handle it separately
        entities.push(EntityType::IpAddress);
        entities
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_detection() {
        let mut detector = PatternDetector::new().unwrap();
        let text = "Contact me at john.doe@example.com or jane@test.org";
        let entities = detector.detect(text).unwrap();

        assert_eq!(entities.len(), 2);
        assert_eq!(entities[0].entity_type, EntityType::Email);
        assert_eq!(entities[0].text, "john.doe@example.com");
        assert_eq!(entities[1].text, "jane@test.org");
    }

    #[test]
    fn test_phone_detection() {
        let mut detector = PatternDetector::new().unwrap();
        let text = "Call me at (555) 123-4567 or 555.987.6543";
        let entities = detector.detect(text).unwrap();

        assert_eq!(entities.len(), 2);
        assert_eq!(entities[0].entity_type, EntityType::PhoneNumber);
        assert_eq!(entities[0].text, "(555) 123-4567");
    }

    #[test]
    fn test_ssn_detection() {
        let mut detector = PatternDetector::new().unwrap();
        let text = "My SSN is 123-45-6789";
        let entities = detector.detect(text).unwrap();

        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].entity_type, EntityType::SocialSecurityNumber);
        assert_eq!(entities[0].text, "123-45-6789");
    }

    #[test]
    fn test_ipv4_detection() {
        let mut detector = PatternDetector::new().unwrap();
        let text = "Server IP: 192.168.1.1 and 10.0.0.1";
        let entities = detector.detect(text).unwrap();

        let ip_entities: Vec<_> = entities
            .iter()
            .filter(|e| e.entity_type == EntityType::IpAddress)
            .collect();

        assert_eq!(ip_entities.len(), 2);
        assert_eq!(ip_entities[0].text, "192.168.1.1");
        assert_eq!(ip_entities[1].text, "10.0.0.1");
    }

    #[test]
    fn test_ipv4_class_detection() {
        let mut detector = PatternDetector::new().unwrap();
        let test_cases = vec![
            ("127.0.0.1", true),       // loopback
            ("192.168.1.100", true),   // private
            ("10.10.10.10", true),     // private
            ("172.16.5.4", true),      // private
            ("8.8.8.8", true),         // public
            ("203.0.113.1", true),     // public
            ("255.255.255.255", true), // broadcast
        ];

        for (ip, should_detect) in test_cases {
            let text = format!("IP address: {}", ip);
            let entities = detector.detect(&text).unwrap();
            let ip_count = entities
                .iter()
                .filter(|e| e.entity_type == EntityType::IpAddress)
                .count();

            if should_detect {
                assert_eq!(ip_count, 1, "Failed to detect valid IPv4: {}", ip);
            } else {
                assert_eq!(ip_count, 0, "Incorrectly detected invalid IPv4: {}", ip);
            }
        }
    }

    #[test]
    fn test_ipv6_detection() {
        let mut detector = PatternDetector::new().unwrap();
        let test_cases = vec![
            "2001:0db8:85a3:0000:0000:8a2e:0370:7334", // full form
            "2001:db8:85a3:0:0:8a2e:370:7334",         // leading zeros omitted
            "2001:db8:85a3::8a2e:370:7334",            // compressed
            "::1",                                     // loopback
            "fe80::1",                                 // link-local
            "::ffff:192.0.2.1",                        // IPv4-mapped IPv6
        ];

        for ipv6 in test_cases {
            let text = format!("IPv6 address: {}", ipv6);
            let entities = detector.detect(&text).unwrap();
            let ip_entities: Vec<_> = entities
                .iter()
                .filter(|e| e.entity_type == EntityType::IpAddress)
                .collect();

            // Debug information
            if ip_entities.len() != 1 {
                eprintln!(
                    "Expected 1 IPv6, got {}: {:?}",
                    ip_entities.len(),
                    ip_entities
                );
                for (i, entity) in ip_entities.iter().enumerate() {
                    eprintln!(
                        "  Entity {}: {} ({}..{})",
                        i, entity.text, entity.start, entity.end
                    );
                }
            }

            assert_eq!(ip_entities.len(), 1, "Failed to detect IPv6: {}", ipv6);
            assert_eq!(ip_entities[0].text, ipv6);
        }
    }

    #[test]
    fn test_malformed_ip_detection() {
        let mut detector = PatternDetector::new().unwrap();
        let test_cases = vec![
            ("192.168.1.1.", false),  // trailing dot
            ("192.168.1..1", false),  // double dot
            ("192.168.1.256", false), // octet > 255
            ("999.168.1.1", false),   // first octet > 255
            ("192.168.1", false),     // incomplete
            ("192.168.1.1.1", false), // too many octets
        ];

        for (malformed_ip, should_detect) in test_cases {
            let text = format!("IP: {}", malformed_ip);
            let entities = detector.detect(&text).unwrap();
            let ip_count = entities
                .iter()
                .filter(|e| e.entity_type == EntityType::IpAddress)
                .count();

            if should_detect {
                assert_eq!(ip_count, 1, "Failed to detect IP: {}", malformed_ip);
            } else {
                assert_eq!(
                    ip_count, 0,
                    "Incorrectly detected malformed IP: {}",
                    malformed_ip
                );
            }
        }
    }

    #[test]
    fn test_cidr_notation_detection() {
        let mut detector = PatternDetector::new().unwrap();

        let test_cases = vec![
            // IPv4 CIDR blocks
            ("192.168.1.0/24", true), // Standard private network
            ("10.0.0.0/8", true),     // Large private network
            ("172.16.0.0/12", true),  // Private network range
            ("8.8.8.0/24", true),     // Public network
            ("127.0.0.0/8", true),    // Loopback network
            // IPv6 CIDR blocks
            ("2001:db8::/32", true),  // Documentation network
            ("fe80::/10", true),      // Link-local network
            ("::1/128", true),        // Loopback host route
            ("2001:4860::/32", true), // Global unicast network
            // Invalid CIDR blocks
            ("192.168.1.0/33", false), // Invalid IPv4 prefix length
            ("192.168.1.0/-1", false), // Negative prefix
            ("192.168.1.0/", false),   // Missing prefix length
            ("2001:db8::/129", false), // Invalid IPv6 prefix length
        ];

        for (cidr, should_detect) in test_cases {
            let text = format!("Network: {}", cidr);
            let entities = detector.detect(&text).unwrap();
            let ip_count = entities
                .iter()
                .filter(|e| e.entity_type == EntityType::IpAddress)
                .count();

            if should_detect {
                assert_eq!(ip_count, 1, "Failed to detect CIDR: {}", cidr);
                let detected_entity = entities
                    .iter()
                    .find(|e| e.entity_type == EntityType::IpAddress)
                    .unwrap();
                assert_eq!(detected_entity.text, cidr, "CIDR text mismatch");
            } else {
                assert_eq!(ip_count, 0, "Incorrectly detected invalid CIDR: {}", cidr);
            }
        }
    }
}
