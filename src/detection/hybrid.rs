use super::ner::GlinerDetector;
use super::patterns::PatternDetector;
use super::{DetectedEntity, EntityDetector, EntityType};
use crate::Result;
use std::collections::HashSet;

pub struct HybridDetector {
    pattern_detector: PatternDetector,
    ner_detector: Option<GlinerDetector>,
    debug_enabled: bool,
    debug_output: String,
}

impl HybridDetector {
    pub fn new() -> Result<Self> {
        let pattern_detector = PatternDetector::new()?;

        // Try to initialize NER detector, but don't fail if models are missing
        let ner_detector = match GlinerDetector::new(
            "models/gliner/gliner_small-v2.1/tokenizer.json",
            "models/gliner/gliner_small-v2.1/model.onnx",
            vec![
                EntityType::Person,
                EntityType::Organization,
                EntityType::Location,
                EntityType::Email,
                EntityType::IpAddress,
            ],
        ) {
            Ok(detector) => Some(detector),
            Err(_) => None,
        };

        Ok(Self {
            pattern_detector,
            ner_detector,
            debug_enabled: false,
            debug_output: String::new(),
        })
    }

    pub fn with_ner_paths(
        tokenizer_path: &str,
        model_path: &str,
        entity_types: Vec<EntityType>,
    ) -> Result<Self> {
        let pattern_detector = PatternDetector::new()?;

        let ner_detector = match GlinerDetector::new(tokenizer_path, model_path, entity_types) {
            Ok(detector) => Some(detector),
            Err(_) => None,
        };

        Ok(Self {
            pattern_detector,
            ner_detector,
            debug_enabled: false,
            debug_output: String::new(),
        })
    }

    pub fn with_model_dir(model_dir: &str) -> Result<Self> {
        let tokenizer_path = format!("{}/tokenizer.json", model_dir);
        let model_path = format!("{}/model.onnx", model_dir);
        Self::with_ner_paths(
            &tokenizer_path,
            &model_path,
            vec![
                EntityType::Person,
                EntityType::Organization,
                EntityType::Location,
                EntityType::Email,
                EntityType::IpAddress,
            ],
        )
    }

    pub fn enable_debug(&mut self, enabled: bool) {
        self.debug_enabled = enabled;
        if enabled {
            self.debug_output.clear();
        }
    }

    pub fn get_debug_output(&self) -> &str {
        &self.debug_output
    }

    fn debug_log(&mut self, message: &str) {
        if self.debug_enabled {
            self.debug_output
                .push_str(&format!("[DEBUG] {}\n", message));
        }
    }

    fn perform_enhanced_ip_detection(
        &mut self,
        text: &str,
        pattern_entities: &[DetectedEntity],
    ) -> Result<Vec<DetectedEntity>> {
        let mut enhanced_entities = Vec::new();

        // Run NER detection if available
        if let Some(ref mut ner_detector) = self.ner_detector {
            match ner_detector.detect(text) {
                Ok(ner_entities) => {
                    // Find NER-detected IP addresses
                    let ner_ips: Vec<_> = ner_entities
                        .iter()
                        .filter(|e| e.entity_type == EntityType::IpAddress)
                        .cloned()
                        .collect();

                    self.debug_log(&format!(
                        "NER found {} IP address candidates: {:?}",
                        ner_ips.len(),
                        ner_ips
                            .iter()
                            .map(|e| &e.text)
                            .collect::<Vec<_>>()
                    ));

                    // Find IP addresses that NER detected but pattern missed
                    let pattern_ip_spans: Vec<(usize, usize)> = pattern_entities
                        .iter()
                        .filter(|e| e.entity_type == EntityType::IpAddress)
                        .map(|e| (e.start, e.end))
                        .collect();

                    for ner_ip in ner_ips {
                        // Check if this NER IP overlaps with any pattern-detected IP
                        let overlaps_with_pattern = pattern_ip_spans.iter().any(|(start, end)| {
                            ner_ip.start < *end && ner_ip.end > *start
                        });

                        if !overlaps_with_pattern {
                            // NER found an IP that pattern missed - this is our enhancement!
                            self.debug_log(&format!(
                                "🎯 NER ENHANCEMENT: Found IP '{}' that pattern missed",
                                ner_ip.text
                            ));
                            enhanced_entities.push(ner_ip);
                        } else {
                            self.debug_log(&format!(
                                "NER IP '{}' overlaps with pattern detection, skipping",
                                ner_ip.text
                            ));
                        }
                    }

                    // Also add non-IP NER entities for completeness
                    let non_ip_ner_entities: Vec<_> = ner_entities
                        .iter()
                        .filter(|e| e.entity_type != EntityType::IpAddress)
                        .cloned()
                        .collect();
                    enhanced_entities.extend(non_ip_ner_entities);
                }
                Err(e) => {
                    self.debug_log(&format!("NER detection failed: {}", e));
                }
            }
        } else {
            self.debug_log("NER detector not available for enhanced IP detection");
        }

        Ok(enhanced_entities)
    }

    fn deduplicate_entities(&mut self, entities: Vec<DetectedEntity>) -> Vec<DetectedEntity> {
        let mut deduped = Vec::new();
        let mut seen_spans = HashSet::new();

        // Sort by start position and confidence (highest first)
        let mut sorted_entities = entities;
        sorted_entities.sort_by(|a, b| {
            a.start.cmp(&b.start).then(
                b.confidence
                    .partial_cmp(&a.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
        });

        for entity in sorted_entities {
            let span = (entity.start, entity.end);

            // Check if this span overlaps with any existing span
            let overlaps = seen_spans.iter().any(|(start, end)| {
                // Two spans overlap if they share any character position
                entity.start < *end && entity.end > *start
            });

            if !overlaps {
                seen_spans.insert(span);
                deduped.push(entity);
            }
        }

        // Sort final result by start position
        deduped.sort_by_key(|e| e.start);
        deduped
    }
}

impl EntityDetector for HybridDetector {
    fn detect(&mut self, text: &str) -> Result<Vec<DetectedEntity>> {
        if self.debug_enabled {
            self.debug_output.clear();
        }

        self.debug_log(&format!("Processing text: '{}'", text));
        let mut all_entities = Vec::new();

        // Always run pattern detection first
        let pattern_entities = self.pattern_detector.detect(text)?;
        self.debug_log(&format!(
            "Pattern detector found {} entities: {:?}",
            pattern_entities.len(),
            pattern_entities
                .iter()
                .map(|e| format!("{}({})", e.entity_type, e.text))
                .collect::<Vec<_>>()
        ));
        all_entities.extend(pattern_entities.clone());

        // Run enhanced IP detection (NER second-pass for missed IPs + regular NER)
        let enhanced_entities = self.perform_enhanced_ip_detection(text, &pattern_entities)?;
        self.debug_log(&format!(
            "Enhanced detection found {} additional entities: {:?}",
            enhanced_entities.len(),
            enhanced_entities
                .iter()
                .map(|e| format!("{}({})", e.entity_type, e.text))
                .collect::<Vec<_>>()
        ));
        all_entities.extend(enhanced_entities);

        // Deduplicate overlapping entities (prefer higher confidence)
        self.debug_log(&format!(
            "Before deduplication: {} total entities",
            all_entities.len()
        ));
        let deduped_entities = self.deduplicate_entities(all_entities);
        self.debug_log(&format!(
            "Final entities after deduplication: {} entities: {:?}",
            deduped_entities.len(),
            deduped_entities
                .iter()
                .map(|e| format!("{}({})", e.entity_type, e.text))
                .collect::<Vec<_>>()
        ));

        Ok(deduped_entities)
    }

    fn supported_entities(&self) -> Vec<EntityType> {
        let mut entities = self.pattern_detector.supported_entities();

        if let Some(ref ner_detector) = self.ner_detector {
            let ner_entities = ner_detector.supported_entities();
            for entity_type in ner_entities {
                if !entities.contains(&entity_type) {
                    entities.push(entity_type);
                }
            }
        }

        entities
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::fmt::Debug for HybridDetector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HybridDetector")
            .field("pattern_detector", &"<PatternDetector>")
            .field("has_ner", &self.ner_detector.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_detector_creation() {
        let detector = HybridDetector::new();
        assert!(detector.is_ok());
    }

    #[test]
    fn test_pattern_only_detection() {
        let mut detector = HybridDetector::new().unwrap();
        let text = "Contact: john@example.com, phone: (555) 123-4567";

        let entities = detector.detect(text).unwrap();

        // Should detect at least email and phone from pattern detection
        let has_email = entities.iter().any(|e| e.entity_type == EntityType::Email);
        let has_phone = entities
            .iter()
            .any(|e| e.entity_type == EntityType::PhoneNumber);

        assert!(has_email, "Should detect email");
        assert!(has_phone, "Should detect phone");
    }

    #[test]
    fn test_deduplication() {
        let mut detector = HybridDetector::new().unwrap();

        // Create overlapping entities to test deduplication
        let entities = vec![
            DetectedEntity::new(
                EntityType::Email,
                "test@example.com".to_string(),
                0,
                16,
                1.0,
            ),
            DetectedEntity::new(EntityType::Person, "test".to_string(), 0, 4, 0.8), // Overlaps with email
        ];

        let deduped = detector.deduplicate_entities(entities);

        // Should keep higher confidence entity (email with 1.0 confidence)
        assert_eq!(deduped.len(), 1);
        assert_eq!(deduped[0].entity_type, EntityType::Email);
    }

    #[test]
    fn test_debug_logging() {
        let mut detector = HybridDetector::new().unwrap();

        // This test should fail initially since debug logging doesn't exist yet
        detector.enable_debug(true);

        let text = "Contact john.smith@microsoft.com for details";
        let _entities = detector.detect(text).unwrap();

        // Should have debug output available
        let debug_output = detector.get_debug_output();
        assert!(debug_output.contains("Pattern detector found"));
        assert!(debug_output.contains("Enhanced detection found"));
        assert!(debug_output.contains("Final entities after deduplication"));
    }

    #[test]
    fn test_enhanced_hybrid_ip_detection() {
        // Test that hybrid detector can detect IPs that pattern matching misses using NER
        let mut detector = match HybridDetector::new() {
            Ok(mut d) => {
                d.enable_debug(true);
                d
            }
            Err(_) => {
                println!("Models not available - skipping enhanced IP detection test");
                return;
            }
        };

        let test_cases = vec![
            // Cases where pattern should work (baseline)
            ("Standard IP", "Server IP: 192.168.1.1", 1),
            ("CIDR block", "Network: 192.168.1.0/24", 1),
            
            // Cases where NER should provide advantage (malformed but semantic)
            ("Wrong separators", "Use IP 192,168,1,1 instead", 1), // NER advantage
            ("Dashes", "Connect to 192-168-1-1", 1),                // NER advantage  
            ("Spaces", "IP: 192. 168. 1. 1", 1),                   // NER advantage
            ("Natural language", "My router 192.168.1.1 crashed", 1), // Both should work
            ("Leading zeros", "Connect to 192.168.001.001", 0),     // May not detect - model limitation
        ];

        for (test_name, text, expected_ip_count) in test_cases {
            let entities = detector.detect(text).expect("Detection should work");
            
            let ip_entities: Vec<_> = entities
                .iter()
                .filter(|e| e.entity_type == EntityType::IpAddress)
                .collect();

            assert_eq!(
                ip_entities.len(),
                expected_ip_count,
                "{}: Expected {} IP(s), got {}. Entities: {:?}",
                test_name,
                expected_ip_count,
                ip_entities.len(),
                ip_entities
            );

            // Verify we found valid IP addresses
            for entity in ip_entities {
                assert!(!entity.text.is_empty(), "IP text should not be empty");
                println!("✅ {} - '{}': Found IP '{}'", test_name, text, entity.text);
            }

            // Optional: Print debug output for analysis
            if detector.debug_enabled {
                println!("Debug output for '{}':\n{}", text, detector.get_debug_output());
            }
        }
    }
}
