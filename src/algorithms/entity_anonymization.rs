use super::AnonymizationAlgorithm;
use crate::detection::{DetectedEntity, EntityDetector, EntityType};
use crate::{AnonError, Dataset, Result};
use rand::rngs::StdRng;
use rand::{Rng, RngCore, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Debug, Clone)]
pub struct EntityAnonymization {
    replacement_strategies: HashMap<EntityType, ReplacementStrategy>,
    preserve_format: bool,
    global_strategy: Option<AnonymizationStrategy>,
    seed: Option<u64>,
    pseudonym_mappings: HashMap<String, String>, // original -> pseudonym
    reverse_mappings: HashMap<String, String>,   // pseudonym -> original
    pools: Option<PseudonymPools>,
}

#[derive(Debug, Clone)]
pub enum ReplacementStrategy {
    Suppress(String),
    Generalize(String),
    Pseudonymize,
    Redact,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnonymizationStrategy {
    Redact,
    Suppress,
    Generalize,
    Pseudonymize,
}

impl std::str::FromStr for AnonymizationStrategy {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "redact" => Ok(AnonymizationStrategy::Redact),
            "suppress" => Ok(AnonymizationStrategy::Suppress),
            "generalize" => Ok(AnonymizationStrategy::Generalize),
            "pseudonymize" => Ok(AnonymizationStrategy::Pseudonymize),
            _ => Err(format!(
                "Invalid strategy: {}. Valid options: redact, suppress, generalize, pseudonymize",
                s
            )),
        }
    }
}

impl std::fmt::Display for AnonymizationStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnonymizationStrategy::Redact => write!(f, "redact"),
            AnonymizationStrategy::Suppress => write!(f, "suppress"),
            AnonymizationStrategy::Generalize => write!(f, "generalize"),
            AnonymizationStrategy::Pseudonymize => write!(f, "pseudonymize"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PseudonymPools {
    pub first_names: Option<Vec<String>>,
    pub last_names: Option<Vec<String>>,
    pub organizations: Option<Vec<String>>,
    pub locations: Option<Vec<String>>,
    pub email_domains: Option<Vec<String>>,
}

impl PseudonymPools {
    pub fn new() -> Self {
        Self {
            first_names: None,
            last_names: None,
            organizations: None,
            locations: None,
            email_domains: None,
        }
    }

    pub fn with_first_names(mut self, names: Vec<String>) -> Self {
        self.first_names = Some(names);
        self
    }

    pub fn with_last_names(mut self, names: Vec<String>) -> Self {
        self.last_names = Some(names);
        self
    }

    pub fn with_organizations(mut self, orgs: Vec<String>) -> Self {
        self.organizations = Some(orgs);
        self
    }

    pub fn with_locations(mut self, locations: Vec<String>) -> Self {
        self.locations = Some(locations);
        self
    }

    pub fn with_email_domains(mut self, domains: Vec<String>) -> Self {
        self.email_domains = Some(domains);
        self
    }

    pub fn generate_with_seed(seed: u64, size: usize) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);

        let first_names = (0..size)
            .map(|i| Self::generate_realistic_first_name(&mut rng, i))
            .collect();

        let last_names = (0..size)
            .map(|i| Self::generate_realistic_last_name(&mut rng, i))
            .collect();

        let organizations = (0..size)
            .map(|i| Self::generate_realistic_organization(&mut rng, i))
            .collect();

        let locations = (0..size)
            .map(|i| Self::generate_realistic_location(&mut rng, i))
            .collect();

        let email_domains = (0..size.min(20)) // Fewer domains needed
            .map(|i| Self::generate_realistic_domain(&mut rng, i))
            .collect();

        Self {
            first_names: Some(first_names),
            last_names: Some(last_names),
            organizations: Some(organizations),
            locations: Some(locations),
            email_domains: Some(email_domains),
        }
    }

    fn generate_realistic_first_name(rng: &mut StdRng, index: usize) -> String {
        let syllables = [
            "Al", "Jo", "Ta", "Ca", "Ri", "Mo", "Bl", "Av", "Ch", "Dr", "Em", "Fi",
        ];
        let endings = ["ex", "an", "or", "ey", "ly", "en", "ke", "ry", "is", "ew"];

        let syl_idx = (rng.next_u32() as usize + index) % syllables.len();
        let end_idx = (rng.next_u32() as usize + index * 2) % endings.len();

        format!("{}{}", syllables[syl_idx], endings[end_idx])
    }

    fn generate_realistic_last_name(rng: &mut StdRng, index: usize) -> String {
        let prefixes = [
            "Sm", "Jo", "Wi", "Br", "Da", "Mi", "Wil", "Mo", "Ta", "An", "Ga", "Ha",
        ];
        let suffixes = [
            "ith", "nson", "liams", "own", "vis", "ller", "son", "ore", "lor", "derson", "rcia",
            "ll",
        ];

        let pre_idx = (rng.next_u32() as usize + index) % prefixes.len();
        let suf_idx = (rng.next_u32() as usize + index * 3) % suffixes.len();

        format!("{}{}", prefixes[pre_idx], suffixes[suf_idx])
    }

    fn generate_realistic_organization(rng: &mut StdRng, index: usize) -> String {
        let prefixes = [
            "Tech", "Data", "Global", "Alpha", "Prime", "Digital", "Future", "Smart", "Core",
            "Next",
        ];
        let suffixes = [
            "Corp",
            "Systems",
            "Solutions",
            "Dynamics",
            "Industries",
            "Labs",
            "Group",
            "Works",
            "Inc",
            "Ltd",
        ];

        let pre_idx = (rng.next_u32() as usize + index) % prefixes.len();
        let suf_idx = (rng.next_u32() as usize + index * 4) % suffixes.len();

        format!("{} {}", prefixes[pre_idx], suffixes[suf_idx])
    }

    fn generate_realistic_location(rng: &mut StdRng, index: usize) -> String {
        let prefixes = [
            "Spring", "River", "Frank", "Mad", "George", "Oak", "Fair", "High", "Mid", "West",
        ];
        let suffixes = [
            "field", "side", "lin", "ison", "town", "ville", "view", "land", "town", "side",
        ];

        let pre_idx = (rng.next_u32() as usize + index) % prefixes.len();
        let suf_idx = (rng.next_u32() as usize + index * 5) % suffixes.len();

        format!("{}{}", prefixes[pre_idx], suffixes[suf_idx])
    }

    fn generate_realistic_domain(rng: &mut StdRng, index: usize) -> String {
        let names = [
            "example",
            "test",
            "demo",
            "sample",
            "placeholder",
            "mock",
            "fake",
        ];
        let tlds = [".com", ".org", ".net", ".co"];

        let name_idx = (rng.next_u32() as usize + index) % names.len();
        let tld_idx = (rng.next_u32() as usize + index * 6) % tlds.len();

        format!("{}{:03}{}", names[name_idx], index, tlds[tld_idx])
    }
}

impl EntityAnonymization {
    pub fn new() -> Self {
        Self {
            replacement_strategies: HashMap::new(),
            preserve_format: true,
            global_strategy: None,
            seed: None,
            pseudonym_mappings: HashMap::new(),
            reverse_mappings: HashMap::new(),
            pools: None,
        }
    }

    pub fn with_preserve_format(mut self, preserve: bool) -> Self {
        self.preserve_format = preserve;
        self
    }

    pub fn add_replacement_strategy(
        &mut self,
        entity_type: EntityType,
        strategy: ReplacementStrategy,
    ) {
        self.replacement_strategies.insert(entity_type, strategy);
    }

    pub fn set_global_strategy(&mut self, strategy: AnonymizationStrategy) {
        self.global_strategy = Some(strategy);
    }

    pub fn with_pools(mut self, pools: PseudonymPools) -> Self {
        self.pools = Some(pools);
        self
    }

    pub fn set_seed(&mut self, seed: u64) {
        self.seed = Some(seed);
        // Clear existing mappings when seed changes
        self.pseudonym_mappings.clear();
        self.reverse_mappings.clear();
    }

    fn get_pools(&self) -> Result<&PseudonymPools> {
        self.pools.as_ref().ok_or_else(|| {
            AnonError::InvalidInput(
                "No pseudonym pools provided. Use with_pools() to set them.".to_string(),
            )
        })
    }

    pub fn get_pseudonym_mapping(&self) -> &HashMap<String, String> {
        &self.pseudonym_mappings
    }

    pub fn reverse_pseudonymization(&self, text: &str) -> Result<String> {
        let mut result = text.to_string();

        // Replace pseudonyms with originals
        for (pseudonym, original) in &self.reverse_mappings {
            result = result.replace(pseudonym, original);
        }

        Ok(result)
    }

    fn get_strategy_for_entity(&self, entity: &DetectedEntity) -> ReplacementStrategy {
        // If global strategy is set, use it for all entities
        if let Some(global_strategy) = &self.global_strategy {
            return self.convert_strategy_for_entity_type(global_strategy, &entity.entity_type);
        }

        // Otherwise, use per-entity strategy or default to Redact
        self.replacement_strategies
            .get(&entity.entity_type)
            .cloned()
            .unwrap_or(ReplacementStrategy::Redact)
    }

    fn convert_strategy_for_entity_type(
        &self,
        strategy: &AnonymizationStrategy,
        entity_type: &EntityType,
    ) -> ReplacementStrategy {
        match strategy {
            AnonymizationStrategy::Redact => ReplacementStrategy::Redact,
            AnonymizationStrategy::Pseudonymize => ReplacementStrategy::Pseudonymize,
            AnonymizationStrategy::Suppress => {
                // Use appropriate suppress format for each entity type
                match entity_type {
                    EntityType::PhoneNumber => {
                        ReplacementStrategy::Suppress("XXX-XXX-XXXX".to_string())
                    }
                    EntityType::Email => ReplacementStrategy::Suppress("[EMAIL]".to_string()),
                    EntityType::SocialSecurityNumber => {
                        ReplacementStrategy::Suppress("XXX-XX-XXXX".to_string())
                    }
                    EntityType::CreditCard => {
                        ReplacementStrategy::Suppress("XXXX-XXXX-XXXX-XXXX".to_string())
                    }
                    EntityType::IpAddress => {
                        ReplacementStrategy::Suppress("XXX.XXX.XXX.XXX".to_string())
                    }
                    EntityType::Person => ReplacementStrategy::Suppress("[PERSON]".to_string()),
                    EntityType::Organization => {
                        ReplacementStrategy::Suppress("[ORGANIZATION]".to_string())
                    }
                    EntityType::Location => ReplacementStrategy::Suppress("[LOCATION]".to_string()),
                    EntityType::Custom(name) => {
                        ReplacementStrategy::Suppress(format!("[{}]", name.to_uppercase()))
                    }
                }
            }
            AnonymizationStrategy::Generalize => {
                // Use appropriate generalize format for each entity type
                match entity_type {
                    EntityType::Email => ReplacementStrategy::Generalize("EMAIL".to_string()),
                    EntityType::PhoneNumber => ReplacementStrategy::Generalize("PHONE".to_string()),
                    EntityType::SocialSecurityNumber => {
                        ReplacementStrategy::Generalize("SSN".to_string())
                    }
                    EntityType::CreditCard => {
                        ReplacementStrategy::Generalize("CREDIT_CARD".to_string())
                    }
                    EntityType::IpAddress => {
                        ReplacementStrategy::Generalize("IP_ADDRESS".to_string())
                    }
                    EntityType::Person => ReplacementStrategy::Generalize("PERSON".to_string()),
                    EntityType::Organization => {
                        ReplacementStrategy::Generalize("ORGANIZATION".to_string())
                    }
                    EntityType::Location => ReplacementStrategy::Generalize("LOCATION".to_string()),
                    EntityType::Custom(name) => {
                        ReplacementStrategy::Generalize(name.to_uppercase())
                    }
                }
            }
        }
    }

    pub fn anonymize_text(
        &mut self,
        text: &str,
        detector: &mut dyn EntityDetector,
    ) -> Result<String> {
        let mut all_entities = detector.detect(text)?;

        all_entities.sort_by_key(|e| e.start);
        all_entities.dedup_by(|a, b| a.start == b.start && a.end == b.end);

        let mut result = text.to_string();
        let mut offset = 0i32;

        for entity in &all_entities {
            let strategy = self.get_strategy_for_entity(entity);

            let replacement = self.generate_replacement(entity, &strategy)?;
            let adjusted_start = (entity.start as i32 + offset) as usize;
            let adjusted_end = (entity.end as i32 + offset) as usize;

            if adjusted_start <= result.len() && adjusted_end <= result.len() {
                result.replace_range(adjusted_start..adjusted_end, &replacement);
                offset += replacement.len() as i32 - entity.length() as i32;
            }
        }

        Ok(result)
    }

    fn generate_replacement(
        &mut self,
        entity: &DetectedEntity,
        strategy: &ReplacementStrategy,
    ) -> Result<String> {
        match strategy {
            ReplacementStrategy::Suppress(replacement) => Ok(replacement.clone()),
            ReplacementStrategy::Generalize(category) => Ok(format!("[{}]", category)),
            ReplacementStrategy::Redact => {
                if self.preserve_format {
                    Ok(self.generate_format_preserving_replacement(entity))
                } else {
                    Ok("[REDACTED]".to_string())
                }
            }
            ReplacementStrategy::Pseudonymize => Ok(self.generate_pseudonym(entity)?),
        }
    }

    fn generate_format_preserving_replacement(&self, entity: &DetectedEntity) -> String {
        match entity.entity_type {
            EntityType::Email => {
                if let Some(at_pos) = entity.text.find('@') {
                    let (local_part, domain_part) = entity.text.split_at(at_pos);
                    format!("{}@{}", "*".repeat(local_part.len()), &domain_part[1..])
                } else {
                    "*".repeat(entity.text.len())
                }
            }
            EntityType::PhoneNumber => entity
                .text
                .chars()
                .map(|c| if c.is_ascii_digit() { '*' } else { c })
                .collect(),
            EntityType::SocialSecurityNumber => entity
                .text
                .chars()
                .map(|c| if c.is_ascii_digit() { '*' } else { c })
                .collect(),
            EntityType::CreditCard => {
                let digits: String = entity.text.chars().filter(|c| c.is_ascii_digit()).collect();
                if digits.len() >= 4 {
                    format!("****-****-****-{}", &digits[digits.len() - 4..])
                } else {
                    "*".repeat(entity.text.len())
                }
            }
            EntityType::Person => {
                // For names, preserve structure (First Last -> ***** ****)
                if entity.text.contains(' ') {
                    entity
                        .text
                        .split_whitespace()
                        .map(|word| "*".repeat(word.len()))
                        .collect::<Vec<_>>()
                        .join(" ")
                } else {
                    "*".repeat(entity.text.len())
                }
            }
            EntityType::Organization => {
                // For organizations, keep some structure
                "*".repeat(entity.text.len().min(10)) + " Corp"
            }
            EntityType::Location => {
                // For locations, just mask
                "*".repeat(entity.text.len())
            }
            _ => "*".repeat(entity.text.len()),
        }
    }

    fn generate_pseudonym(&mut self, entity: &DetectedEntity) -> Result<String> {
        // Check if we already have a pseudonym for this original value
        let original_text = &entity.text;
        if let Some(existing_pseudonym) = self.pseudonym_mappings.get(original_text) {
            return Ok(existing_pseudonym.clone());
        }

        // Create deterministic RNG from seed + original text hash
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};

        // Combine seed (if set) with the original text for deterministic generation
        if let Some(seed) = self.seed {
            seed.hash(&mut hasher);
        }
        original_text.hash(&mut hasher);
        entity.entity_type.hash(&mut hasher);

        let deterministic_seed = hasher.finish();
        let mut rng = StdRng::seed_from_u64(deterministic_seed);

        // Get pools if needed (not required for IP addresses which use algorithmic generation)
        let pools = if entity.entity_type == EntityType::IpAddress
            || entity.entity_type == EntityType::PhoneNumber
            || entity.entity_type == EntityType::SocialSecurityNumber
            || entity.entity_type == EntityType::CreditCard
        {
            None
        } else {
            Some(self.get_pools()?)
        };

        let pseudonym = match entity.entity_type {
            EntityType::Person => {
                let pools = pools.as_ref().unwrap();
                let first_names = pools.first_names.as_ref().ok_or_else(|| {
                    AnonError::InvalidInput("No first names pool provided".to_string())
                })?;
                let last_names = pools.last_names.as_ref().ok_or_else(|| {
                    AnonError::InvalidInput("No last names pool provided".to_string())
                })?;

                if first_names.is_empty() || last_names.is_empty() {
                    return Err(AnonError::InvalidInput(
                        "Empty name pools provided".to_string(),
                    ));
                }

                format!(
                    "{} {}",
                    first_names[rng.gen_range(0..first_names.len())],
                    last_names[rng.gen_range(0..last_names.len())]
                )
            }
            EntityType::Email => {
                let pools = pools.as_ref().unwrap();
                let domains = pools.email_domains.as_ref().ok_or_else(|| {
                    AnonError::InvalidInput("No email domains pool provided".to_string())
                })?;

                if domains.is_empty() {
                    return Err(AnonError::InvalidInput(
                        "Empty email domains pool provided".to_string(),
                    ));
                }

                format!(
                    "user{}@{}",
                    rng.gen_range(1000..9999),
                    domains[rng.gen_range(0..domains.len())]
                )
            }
            EntityType::PhoneNumber => {
                // Phone numbers don't need pools - use algorithmic generation
                format!(
                    "555-{:03}-{:04}",
                    rng.gen_range(100..999),
                    rng.gen_range(1000..9999)
                )
            }
            EntityType::Organization => {
                let pools = pools.as_ref().unwrap();
                let organizations = pools.organizations.as_ref().ok_or_else(|| {
                    AnonError::InvalidInput("No organizations pool provided".to_string())
                })?;

                if organizations.is_empty() {
                    return Err(AnonError::InvalidInput(
                        "Empty organizations pool provided".to_string(),
                    ));
                }

                organizations[rng.gen_range(0..organizations.len())].clone()
            }
            EntityType::Location => {
                let pools = pools.as_ref().unwrap();
                let locations = pools.locations.as_ref().ok_or_else(|| {
                    AnonError::InvalidInput("No locations pool provided".to_string())
                })?;

                if locations.is_empty() {
                    return Err(AnonError::InvalidInput(
                        "Empty locations pool provided".to_string(),
                    ));
                }

                locations[rng.gen_range(0..locations.len())].clone()
            }
            EntityType::SocialSecurityNumber => {
                // SSN doesn't need pools - use algorithmic generation
                format!(
                    "{:03}-{:02}-{:04}",
                    rng.gen_range(100..999),
                    rng.gen_range(10..99),
                    rng.gen_range(1000..9999)
                )
            }
            EntityType::CreditCard => {
                // Credit cards don't need pools - use algorithmic generation
                format!("****-****-****-{:04}", rng.gen_range(1000..9999))
            }
            EntityType::IpAddress => {
                // IP addresses use algorithmic generation with class preservation
                self.generate_class_preserving_ip_pseudonym(original_text, &mut rng)?
            }
            _ => format!("[PSEUDO_{}]", entity.entity_type),
        };

        // Store mappings for consistency and reversibility
        self.pseudonym_mappings
            .insert(original_text.clone(), pseudonym.clone());
        self.reverse_mappings
            .insert(pseudonym.clone(), original_text.clone());

        Ok(pseudonym)
    }

    fn generate_class_preserving_ip_pseudonym(
        &self,
        original_ip: &str,
        rng: &mut StdRng,
    ) -> Result<String> {
        // Check if it's CIDR notation first
        if let Some(slash_pos) = original_ip.find('/') {
            let (ip_part, prefix_part) = original_ip.split_at(slash_pos);
            let prefix_str = &prefix_part[1..]; // Skip the '/' character
            
            // Validate and preserve prefix length
            let prefix_len = prefix_str.parse::<u8>().map_err(|_| {
                AnonError::InvalidInput(format!(
                    "Invalid CIDR prefix length: {}",
                    prefix_str
                ))
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
                self.generate_ipv4_pseudonym(ipv4, rng)
            } else if let Ok(ipv6) = ip_part.parse::<Ipv6Addr>() {
                // Validate IPv6 prefix length
                if prefix_len > 128 {
                    return Err(AnonError::InvalidInput(format!(
                        "Invalid IPv6 prefix length: /{}",
                        prefix_len
                    )));
                }
                self.generate_ipv6_pseudonym(ipv6, rng)
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
            return Ok(self.generate_ipv4_pseudonym(ipv4, rng));
        }

        // Try parsing as IPv6
        if let Ok(ipv6) = original_ip.parse::<Ipv6Addr>() {
            return Ok(self.generate_ipv6_pseudonym(ipv6, rng));
        }

        // Fallback for invalid IP addresses
        Err(AnonError::InvalidInput(format!(
            "Invalid IP address format: {}",
            original_ip
        )))
    }

    fn generate_ipv4_pseudonym(&self, original: Ipv4Addr, rng: &mut StdRng) -> String {
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
        } else if self.is_ipv4_private(octets) {
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
                if !self.is_ipv4_private(candidate) && a != 127 {
                    return format!("{}.{}.{}.{}", a, b, c, d);
                }
            }
        }
    }

    fn generate_ipv6_pseudonym(&self, original: Ipv6Addr, rng: &mut StdRng) -> String {
        let segments = original.segments();

        if original.is_loopback() {
            // IPv6 loopback should remain ::1
            "::1".to_string()
        } else if original.is_unspecified() {
            // IPv6 unspecified should remain ::
            "::".to_string()
        } else if self.is_ipv6_link_local(segments) {
            // Link-local: fe80::/10
            format!(
                "fe80::{:x}:{:x}:{:x}:{:x}",
                rng.gen_range(0..=0xffff),
                rng.gen_range(0..=0xffff),
                rng.gen_range(0..=0xffff),
                rng.gen_range(1..=0xffff)
            )
        } else if self.is_ipv6_documentation(segments) {
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
        } else if self.is_ipv4_mapped_ipv6(segments) {
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

    fn is_ipv4_private(&self, octets: [u8; 4]) -> bool {
        match octets[0] {
            10 => true,                                    // 10.0.0.0/8
            172 if (16..=31).contains(&octets[1]) => true, // 172.16.0.0/12
            192 if octets[1] == 168 => true,               // 192.168.0.0/16
            _ => false,
        }
    }

    fn is_ipv6_link_local(&self, segments: [u16; 8]) -> bool {
        (segments[0] & 0xffc0) == 0xfe80 // fe80::/10
    }

    fn is_ipv6_documentation(&self, segments: [u16; 8]) -> bool {
        segments[0] == 0x2001 && segments[1] == 0x0db8 // 2001:db8::/32
    }

    fn is_ipv4_mapped_ipv6(&self, segments: [u16; 8]) -> bool {
        segments[0..5] == [0, 0, 0, 0, 0] && segments[5] == 0xffff // ::ffff:0:0/96
    }
}

impl AnonymizationAlgorithm for EntityAnonymization {
    fn anonymize(&self, dataset: &Dataset) -> Result<Dataset> {
        self.validate_parameters()?;

        let anonymized = dataset.clone();

        Ok(anonymized)
    }

    fn validate_parameters(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detection::patterns::PatternDetector;

    #[test]
    fn test_text_anonymization() {
        let mut anonymizer = EntityAnonymization::new();

        anonymizer.add_replacement_strategy(EntityType::Email, ReplacementStrategy::Redact);

        let mut detector = PatternDetector::new().unwrap();
        let text = "Contact John at john.doe@example.com for more info.";

        let result = anonymizer.anonymize_text(text, &mut detector).unwrap();
        assert!(result.contains("*****@example.com"));
    }

    #[test]
    fn test_pseudonymization() {
        let pools = PseudonymPools::generate_with_seed(42, 50);
        let mut anonymizer = EntityAnonymization::new().with_pools(pools);

        anonymizer.add_replacement_strategy(EntityType::Person, ReplacementStrategy::Pseudonymize);

        let entity = DetectedEntity::new(EntityType::Person, "John Doe".to_string(), 0, 8, 0.9);

        let strategy = &ReplacementStrategy::Pseudonymize;
        let result = anonymizer.generate_replacement(&entity, strategy).unwrap();

        assert!(result.contains(" "), "Pseudonym should contain a space");
        assert_ne!(
            result, "John Doe",
            "Pseudonym should be different from original"
        );
        assert!(!result.is_empty(), "Pseudonym should not be empty");
    }

    #[test]
    fn test_ipv4_pseudonymization_class_preservation() {
        let mut anonymizer = EntityAnonymization::new();
        anonymizer.set_seed(42);
        anonymizer
            .add_replacement_strategy(EntityType::IpAddress, ReplacementStrategy::Pseudonymize);

        let test_cases = vec![
            ("127.0.0.1", "loopback"),  // Should remain loopback (127.x.x.x)
            ("192.168.1.1", "private"), // Should remain private (192.168.x.x, 10.x.x.x, or 172.16-31.x.x)
            ("10.0.0.1", "private"),    // Should remain private
            ("172.16.5.4", "private"),  // Should remain private
            ("8.8.8.8", "public"),      // Should remain public
            ("203.0.113.1", "public"),  // Should remain public
        ];

        for (original_ip, ip_class) in test_cases {
            let entity = DetectedEntity::new(
                EntityType::IpAddress,
                original_ip.to_string(),
                0,
                original_ip.len(),
                1.0,
            );

            let strategy = &ReplacementStrategy::Pseudonymize;
            let pseudonym = anonymizer.generate_replacement(&entity, strategy).unwrap();

            // Verify pseudonym is different from original
            assert_ne!(
                pseudonym, original_ip,
                "Pseudonym should differ from original"
            );

            // Verify pseudonym preserves IP class
            match ip_class {
                "loopback" => {
                    assert!(
                        pseudonym.starts_with("127."),
                        "Loopback IP {} should remain in 127.x.x.x range, got {}",
                        original_ip,
                        pseudonym
                    );
                }
                "private" => {
                    let is_private = pseudonym.starts_with("192.168.")
                        || pseudonym.starts_with("10.")
                        || (pseudonym.starts_with("172.") && {
                            let parts: Vec<&str> = pseudonym.split('.').collect();
                            if parts.len() >= 2 {
                                if let Ok(second_octet) = parts[1].parse::<u8>() {
                                    (16..=31).contains(&second_octet)
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        });
                    assert!(
                        is_private,
                        "Private IP {} should remain private, got {}",
                        original_ip, pseudonym
                    );
                }
                "public" => {
                    let is_public = !pseudonym.starts_with("127.")
                        && !pseudonym.starts_with("192.168.")
                        && !pseudonym.starts_with("10.")
                        && !(pseudonym.starts_with("172.") && {
                            let parts: Vec<&str> = pseudonym.split('.').collect();
                            if parts.len() >= 2 {
                                if let Ok(second_octet) = parts[1].parse::<u8>() {
                                    (16..=31).contains(&second_octet)
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        });
                    assert!(
                        is_public,
                        "Public IP {} should remain public, got {}",
                        original_ip, pseudonym
                    );
                }
                _ => panic!("Unknown IP class: {}", ip_class),
            }

            // Verify consistency - same input should produce same output
            let second_pseudonym = anonymizer.generate_replacement(&entity, strategy).unwrap();
            assert_eq!(
                pseudonym, second_pseudonym,
                "Pseudonymization should be deterministic"
            );
        }
    }

    #[test]
    fn test_ipv6_pseudonymization_class_preservation() {
        let mut anonymizer = EntityAnonymization::new();
        anonymizer.set_seed(42);
        anonymizer
            .add_replacement_strategy(EntityType::IpAddress, ReplacementStrategy::Pseudonymize);

        let test_cases = vec![
            ("::1", "loopback"),                 // IPv6 loopback
            ("fe80::1", "link_local"),           // Link-local
            ("2001:db8::1", "documentation"),    // Documentation prefix
            ("2001:4860:4860::8888", "global"),  // Global unicast
            ("::ffff:192.0.2.1", "ipv4_mapped"), // IPv4-mapped IPv6
        ];

        for (original_ip, ip_class) in test_cases {
            let entity = DetectedEntity::new(
                EntityType::IpAddress,
                original_ip.to_string(),
                0,
                original_ip.len(),
                1.0,
            );

            let strategy = &ReplacementStrategy::Pseudonymize;
            let pseudonym = anonymizer.generate_replacement(&entity, strategy).unwrap();

            // Verify pseudonym behavior based on special cases
            match ip_class {
                "loopback" => {
                    // IPv6 loopback ::1 should remain ::1 for semantic meaning
                    if original_ip == "::1" {
                        assert_eq!(
                            pseudonym, "::1",
                            "IPv6 loopback ::1 should remain unchanged for semantic meaning"
                        );
                    } else {
                        assert_ne!(
                            pseudonym, original_ip,
                            "Pseudonym should differ from original"
                        );
                    }
                }
                _ => {
                    assert_ne!(
                        pseudonym, original_ip,
                        "Pseudonym should differ from original"
                    );
                }
            }

            // Verify pseudonym preserves IPv6 class
            match ip_class {
                "loopback" => {
                    assert_eq!(
                        pseudonym, "::1",
                        "IPv6 loopback should remain ::1, got {}",
                        pseudonym
                    );
                }
                "link_local" => {
                    assert!(
                        pseudonym.starts_with("fe80::"),
                        "Link-local IPv6 {} should remain in fe80:: range, got {}",
                        original_ip,
                        pseudonym
                    );
                }
                "documentation" => {
                    assert!(
                        pseudonym.starts_with("2001:db8::"),
                        "Documentation IPv6 {} should remain in 2001:db8:: range, got {}",
                        original_ip,
                        pseudonym
                    );
                }
                "global" => {
                    let is_global = !pseudonym.starts_with("::1")
                        && !pseudonym.starts_with("fe80::")
                        && !pseudonym.starts_with("2001:db8::")
                        && !pseudonym.starts_with("::ffff:");
                    assert!(
                        is_global,
                        "Global IPv6 {} should remain global, got {}",
                        original_ip, pseudonym
                    );
                }
                "ipv4_mapped" => {
                    assert!(
                        pseudonym.starts_with("::ffff:"),
                        "IPv4-mapped IPv6 {} should remain IPv4-mapped, got {}",
                        original_ip,
                        pseudonym
                    );
                }
                _ => panic!("Unknown IPv6 class: {}", ip_class),
            }
        }
    }

    #[test]
    fn test_cidr_pseudonymization_semantic_preservation() {
        let mut anonymizer = EntityAnonymization::new();
        anonymizer.set_seed(42);
        anonymizer.add_replacement_strategy(EntityType::IpAddress, ReplacementStrategy::Pseudonymize);

        let test_cases = vec![
            // IPv4 CIDR blocks
            ("192.168.1.0/24", "private", 24),      // Private /24 network
            ("10.0.0.0/8", "private", 8),           // Private /8 network
            ("172.16.0.0/12", "private", 12),       // Private /12 network
            ("8.8.8.0/24", "public", 24),           // Public /24 network
            ("127.0.0.0/8", "loopback", 8),         // Loopback /8 network
            
            // IPv6 CIDR blocks
            ("2001:db8::/32", "documentation", 32), // Documentation /32 network
            ("fe80::/10", "link_local", 10),        // Link-local /10 network
            ("::1/128", "loopback", 128),           // IPv6 loopback host route
            ("2001:4860::/32", "global", 32),       // Global unicast /32
        ];

        for (original_cidr, ip_class, expected_prefix) in test_cases {
            let entity = DetectedEntity::new(
                EntityType::IpAddress,
                original_cidr.to_string(),
                0,
                original_cidr.len(),
                1.0,
            );

            let strategy = &ReplacementStrategy::Pseudonymize;
            let pseudonym = anonymizer.generate_replacement(&entity, strategy).unwrap();

            // Verify pseudonym is different from original (except special cases)
            match ip_class {
                "loopback" => {
                    if original_cidr == "::1/128" {
                        assert_eq!(pseudonym, "::1/128", 
                            "IPv6 loopback host route should remain unchanged");
                    } else {
                        // IPv4 loopback should stay in 127.x.x.x range with same prefix
                        assert!(pseudonym.starts_with("127."), 
                            "Loopback CIDR {} should remain in 127.x.x.x range, got {}", 
                            original_cidr, pseudonym);
                        assert!(pseudonym.ends_with(&format!("/{}", expected_prefix)),
                            "Prefix length should be preserved: expected /{}, got {}", 
                            expected_prefix, pseudonym);
                    }
                }
                _ => {
                    assert_ne!(pseudonym, original_cidr, 
                        "CIDR pseudonym should differ from original");
                }
            }

            // Verify CIDR format preservation
            if let Some(slash_pos) = pseudonym.find('/') {
                let (_, prefix_part) = pseudonym.split_at(slash_pos);
                let prefix_str = &prefix_part[1..];
                let actual_prefix: u8 = prefix_str.parse().expect("Prefix should be valid number");
                
                assert_eq!(actual_prefix, expected_prefix, 
                    "Prefix length should be preserved: expected /{}, got /{} in {}", 
                    expected_prefix, actual_prefix, pseudonym);
            } else {
                panic!("Pseudonym should preserve CIDR format: got '{}' for '{}'", 
                    pseudonym, original_cidr);
            }

            // Verify IP class preservation
            let (pseudo_ip_part, _) = pseudonym.split_at(pseudonym.find('/').unwrap());
            
            match ip_class {
                "private" => {
                    let is_private = pseudo_ip_part.starts_with("192.168.") ||
                                   pseudo_ip_part.starts_with("10.") ||
                                   (pseudo_ip_part.starts_with("172.") && {
                                       let parts: Vec<&str> = pseudo_ip_part.split('.').collect();
                                       if parts.len() >= 2 {
                                           if let Ok(second_octet) = parts[1].parse::<u8>() {
                                               (16..=31).contains(&second_octet)
                                           } else { false }
                                       } else { false }
                                   });
                    assert!(is_private, 
                        "Private CIDR {} should remain private, got {}", 
                        original_cidr, pseudonym);
                }
                "public" => {
                    let is_public = !pseudo_ip_part.starts_with("127.") &&
                                  !pseudo_ip_part.starts_with("192.168.") &&
                                  !pseudo_ip_part.starts_with("10.") &&
                                  !(pseudo_ip_part.starts_with("172.") && {
                                      let parts: Vec<&str> = pseudo_ip_part.split('.').collect();
                                      if parts.len() >= 2 {
                                          if let Ok(second_octet) = parts[1].parse::<u8>() {
                                              (16..=31).contains(&second_octet)
                                          } else { false }
                                      } else { false }
                                  });
                    assert!(is_public, 
                        "Public CIDR {} should remain public, got {}", 
                        original_cidr, pseudonym);
                }
                "documentation" => {
                    assert!(pseudo_ip_part.starts_with("2001:db8"), 
                        "Documentation CIDR {} should remain in 2001:db8:: range, got {}", 
                        original_cidr, pseudonym);
                }
                "link_local" => {
                    assert!(pseudo_ip_part.starts_with("fe80"), 
                        "Link-local CIDR {} should remain in fe80:: range, got {}", 
                        original_cidr, pseudonym);
                }
                "global" => {
                    let is_global = !pseudo_ip_part.starts_with("::1") &&
                                  !pseudo_ip_part.starts_with("fe80") &&
                                  !pseudo_ip_part.starts_with("2001:db8") &&
                                  !pseudo_ip_part.starts_with("::ffff:");
                    assert!(is_global, 
                        "Global CIDR {} should remain global, got {}", 
                        original_cidr, pseudonym);
                }
                _ => {}
            }

            // Verify consistency
            let second_pseudonym = anonymizer.generate_replacement(&entity, strategy).unwrap();
            assert_eq!(pseudonym, second_pseudonym, 
                "CIDR pseudonymization should be deterministic");
        }
    }
}
