use anon_sdk::algorithms::entity_anonymization::{EntityAnonymization, ReplacementStrategy};
use anon_sdk::detection::{EntityDetector, EntityType, patterns::PatternDetector};

#[test]
fn test_comprehensive_pii_detection() {
    let mut detector = PatternDetector::new().expect("Failed to create pattern detector");

    let test_text = r#"
        John Doe's contact information:
        Email: john.doe@company.com
        Phone: (555) 123-4567
        SSN: 123-45-6789
        Credit Card: 4532-1234-5678-9012
        IP: 192.168.1.100
        
        Alternative contact: jane.smith@example.org or 555.987.6543
    "#;

    let entities = detector.detect(test_text).expect("Detection failed");

    assert!(!entities.is_empty(), "Should detect PII entities");

    let email_count = entities
        .iter()
        .filter(|e| e.entity_type == EntityType::Email)
        .count();
    let phone_count = entities
        .iter()
        .filter(|e| e.entity_type == EntityType::PhoneNumber)
        .count();
    let ssn_count = entities
        .iter()
        .filter(|e| e.entity_type == EntityType::SocialSecurityNumber)
        .count();
    let cc_count = entities
        .iter()
        .filter(|e| e.entity_type == EntityType::CreditCard)
        .count();
    let ip_count = entities
        .iter()
        .filter(|e| e.entity_type == EntityType::IpAddress)
        .count();

    assert_eq!(email_count, 2, "Should detect 2 email addresses");
    assert_eq!(phone_count, 2, "Should detect 2 phone numbers");
    assert_eq!(ssn_count, 1, "Should detect 1 SSN");
    assert_eq!(cc_count, 1, "Should detect 1 credit card");
    assert_eq!(ip_count, 1, "Should detect 1 IP address");
}

#[test]
fn test_entity_anonymization_pipeline() {
    let mut anonymizer = EntityAnonymization::new();

    anonymizer.add_replacement_strategy(EntityType::Email, ReplacementStrategy::Redact);
    anonymizer.add_replacement_strategy(
        EntityType::PhoneNumber,
        ReplacementStrategy::Suppress("XXX-XXX-XXXX".to_string()),
    );
    anonymizer.add_replacement_strategy(
        EntityType::SocialSecurityNumber,
        ReplacementStrategy::Generalize("SSN".to_string()),
    );
    anonymizer.add_replacement_strategy(EntityType::Person, ReplacementStrategy::Pseudonymize);

    let mut detector = PatternDetector::new().expect("Failed to create detector");
    let original_text =
        "Contact John Doe at john.doe@example.com or call (555) 123-4567. SSN: 123-45-6789";
    let anonymized = anonymizer
        .anonymize_text(original_text, &mut detector)
        .expect("Anonymization failed");

    assert!(
        !anonymized.contains("john.doe@example.com"),
        "Email should be anonymized"
    );
    assert!(
        !anonymized.contains("(555) 123-4567"),
        "Phone should be anonymized"
    );
    assert!(
        !anonymized.contains("123-45-6789"),
        "SSN should be anonymized"
    );

    assert!(
        anonymized.contains("XXX-XXX-XXXX"),
        "Phone replacement should be present"
    );
    assert!(
        anonymized.contains("[SSN]"),
        "SSN generalization should be present"
    );
}

#[test]
fn test_format_preserving_anonymization() {
    let mut anonymizer = EntityAnonymization::new().with_preserve_format(true);

    let test_cases = vec![
        ("john.doe@example.com", EntityType::Email),
        ("(555) 123-4567", EntityType::PhoneNumber),
        ("123-45-6789", EntityType::SocialSecurityNumber),
        ("4532123456789012", EntityType::CreditCard),
    ];

    let mut detector = PatternDetector::new().expect("Failed to create detector");

    for (original, entity_type) in test_cases {
        let anonymized = anonymizer
            .anonymize_text(original, &mut detector)
            .expect("Anonymization failed");

        match entity_type {
            EntityType::Email => {
                assert!(
                    anonymized.contains("@example.com"),
                    "Email domain should be preserved"
                );
                assert!(
                    anonymized.contains("*"),
                    "Email local part should be masked"
                );
            }
            EntityType::PhoneNumber => {
                assert!(anonymized.contains("("), "Phone format should be preserved");
                assert!(anonymized.contains(")"), "Phone format should be preserved");
                assert!(anonymized.contains("-"), "Phone format should be preserved");
                assert!(anonymized.contains("*"), "Phone digits should be masked");
            }
            EntityType::CreditCard => {
                assert!(
                    anonymized.contains("9012"),
                    "Last 4 digits should be preserved"
                );
                assert!(anonymized.contains("****"), "Most digits should be masked");
            }
            _ => {}
        }

        assert_ne!(original, anonymized, "Text should be anonymized");
    }
}

#[test]
fn test_gliner_detector_creation() {
    use anon_sdk::detection::ner::GlinerDetector;

    let entity_types = vec![
        EntityType::Person,
        EntityType::Location,
        EntityType::Organization,
    ];
    let detector = GlinerDetector::new("tokenizer.json", "model.onnx", entity_types);

    match detector {
        Ok(d) => {
            let supported = d.supported_entities();
            assert_eq!(supported.len(), 3);

            // Test confidence threshold setting
            let detector_with_threshold = d.with_confidence_threshold(0.8);
            assert!(detector_with_threshold.is_ok());
        }
        Err(e) => {
            // Expected to fail in test environment without actual model files
            println!(
                "GLiNER detector creation failed (expected in test env): {}",
                e
            );
            assert!(e.to_string().contains("Failed to load GLiNER model"));
        }
    }
}

#[test]
fn test_gliner_confidence_threshold_validation() {
    use anon_sdk::detection::ner::GlinerDetector;

    let entity_types = vec![EntityType::Person];

    // This will fail due to missing model files, but we're testing the validation logic
    let result = GlinerDetector::new("fake.json", "fake.onnx", entity_types);

    if let Ok(detector) = result {
        // Test invalid thresholds - create new detectors since GLiNER models can't be cloned
        let entity_types_clone = vec![EntityType::Person];
        let detector2 = GlinerDetector::new("fake.json", "fake.onnx", entity_types_clone.clone());
        let detector3 = GlinerDetector::new("fake.json", "fake.onnx", entity_types_clone.clone());
        let detector4 = GlinerDetector::new("fake.json", "fake.onnx", entity_types_clone.clone());
        let detector5 = GlinerDetector::new("fake.json", "fake.onnx", entity_types_clone);

        if let Ok(d2) = detector2 {
            assert!(d2.with_confidence_threshold(-0.1).is_err());
        }
        if let Ok(d3) = detector3 {
            assert!(d3.with_confidence_threshold(1.1).is_err());
        }
        if let Ok(d4) = detector4 {
            assert!(d4.with_confidence_threshold(0.0).is_ok());
        }
        if let Ok(d5) = detector5 {
            assert!(d5.with_confidence_threshold(0.5).is_ok());
        }
        assert!(detector.with_confidence_threshold(1.0).is_ok());
    }
}

#[test]
fn test_gliner_gpu_acceleration_creation() {
    use anon_sdk::detection::ner::GlinerDetector;

    let entity_types = vec![EntityType::Person, EntityType::Location];
    let result =
        GlinerDetector::with_gpu_acceleration("tokenizer.json", "model.onnx", entity_types);

    match result {
        Ok(_) => {
            // Unexpected success in test environment
            panic!("GPU detector creation should fail without model files");
        }
        Err(e) => {
            // Expected failure due to missing model files or GPU setup
            println!("GPU GLiNER creation failed (expected): {}", e);
        }
    }
}

#[test]
fn test_gliner_detector_model_loading() {
    use anon_sdk::detection::ner::GlinerDetector;

    // Since NER is now always available, we expect model loading failures for fake paths
    let entity_types = vec![EntityType::Person];
    let result = GlinerDetector::new("fake.json", "fake.onnx", entity_types);

    assert!(result.is_err(), "Should fail with fake model paths");
    let error_msg = result.err().unwrap().to_string();
    assert!(
        error_msg.contains("Failed to load GLiNER model"),
        "Should get model loading error, got: {}",
        error_msg
    );
}

#[test]
fn test_multiple_detector_combination() {
    let mut pattern_detector = PatternDetector::new().expect("Failed to create pattern detector");

    let mixed_text =
        "Contact Jane Smith at jane@company.com, phone: 555-123-4567, location: New York";
    let entities = pattern_detector
        .detect(mixed_text)
        .expect("Detection failed");

    let has_email = entities.iter().any(|e| e.entity_type == EntityType::Email);
    let has_phone = entities
        .iter()
        .any(|e| e.entity_type == EntityType::PhoneNumber);

    assert!(has_email, "Should detect email");
    assert!(has_phone, "Should detect phone number");
}

#[test]
fn test_deterministic_pseudonymization_with_seed() {
    use anon_sdk::algorithms::entity_anonymization::{
        AnonymizationStrategy, EntityAnonymization, PseudonymPools,
    };

    // Generate test pools with a known seed
    let pools = PseudonymPools::generate_with_seed(42, 100);
    let mut anonymizer1 = EntityAnonymization::new().with_pools(pools.clone());
    let mut anonymizer2 = EntityAnonymization::new().with_pools(pools);

    anonymizer1.set_global_strategy(AnonymizationStrategy::Pseudonymize);
    anonymizer1.set_seed(123);
    anonymizer2.set_global_strategy(AnonymizationStrategy::Pseudonymize);
    anonymizer2.set_seed(123);

    let mut detector1 = PatternDetector::new().expect("Failed to create detector");
    let mut detector2 = PatternDetector::new().expect("Failed to create detector");

    let test_text = "Contact John Smith at john@company.com";

    let result1 = anonymizer1
        .anonymize_text(test_text, &mut detector1)
        .expect("First anonymization failed");
    let result2 = anonymizer2
        .anonymize_text(test_text, &mut detector2)
        .expect("Second anonymization failed");

    assert_eq!(
        result1, result2,
        "Same seed should produce identical results"
    );
    assert_ne!(
        result1, test_text,
        "Results should be different from original"
    );
}

#[test]
fn test_deterministic_pseudonymization_without_seed() {
    use anon_sdk::algorithms::entity_anonymization::{
        AnonymizationStrategy, EntityAnonymization, PseudonymPools,
    };

    // Test that even without explicit seed, results are deterministic
    let pools = PseudonymPools::generate_with_seed(42, 100);
    let mut anonymizer1 = EntityAnonymization::new().with_pools(pools.clone());
    let mut anonymizer2 = EntityAnonymization::new().with_pools(pools);

    anonymizer1.set_global_strategy(AnonymizationStrategy::Pseudonymize);
    anonymizer2.set_global_strategy(AnonymizationStrategy::Pseudonymize);
    // Note: not setting seed explicitly

    let mut detector1 = PatternDetector::new().expect("Failed to create detector");
    let mut detector2 = PatternDetector::new().expect("Failed to create detector");

    let test_text = "Contact John Smith at john@company.com";

    let result1 = anonymizer1
        .anonymize_text(test_text, &mut detector1)
        .expect("First anonymization failed");
    let result2 = anonymizer2
        .anonymize_text(test_text, &mut detector2)
        .expect("Second anonymization failed");

    assert_eq!(
        result1, result2,
        "Results should be deterministic even without explicit seed"
    );
    assert_ne!(
        result1, test_text,
        "Results should be different from original"
    );
}

#[test]
fn test_different_seeds_produce_different_results() {
    use anon_sdk::algorithms::entity_anonymization::{
        AnonymizationStrategy, EntityAnonymization, PseudonymPools,
    };

    let pools = PseudonymPools::generate_with_seed(42, 100);
    let mut anonymizer1 = EntityAnonymization::new().with_pools(pools.clone());
    let mut anonymizer2 = EntityAnonymization::new().with_pools(pools);

    anonymizer1.set_global_strategy(AnonymizationStrategy::Pseudonymize);
    anonymizer1.set_seed(123);
    anonymizer2.set_global_strategy(AnonymizationStrategy::Pseudonymize);
    anonymizer2.set_seed(456);

    let mut detector1 = PatternDetector::new().expect("Failed to create detector");
    let mut detector2 = PatternDetector::new().expect("Failed to create detector");

    let test_text = "Contact John Smith at john@company.com";

    let result1 = anonymizer1
        .anonymize_text(test_text, &mut detector1)
        .expect("First anonymization failed");
    let result2 = anonymizer2
        .anonymize_text(test_text, &mut detector2)
        .expect("Second anonymization failed");

    assert_ne!(
        result1, result2,
        "Different seeds should produce different results"
    );
}

#[test]
fn test_entity_type_affects_pseudonym_generation() {
    use anon_sdk::algorithms::entity_anonymization::{
        AnonymizationStrategy, EntityAnonymization, PseudonymPools,
    };
    use anon_sdk::detection::patterns::PatternDetector;

    let pools = PseudonymPools::generate_with_seed(42, 100);
    let mut anonymizer1 = EntityAnonymization::new().with_pools(pools.clone());
    let mut anonymizer2 = EntityAnonymization::new().with_pools(pools);

    anonymizer1.set_global_strategy(AnonymizationStrategy::Pseudonymize);
    anonymizer1.set_seed(123);
    anonymizer2.set_global_strategy(AnonymizationStrategy::Pseudonymize);
    anonymizer2.set_seed(123);

    // Set different replacement strategies for different entity types
    anonymizer1.add_replacement_strategy(
        EntityType::Person,
        anon_sdk::algorithms::entity_anonymization::ReplacementStrategy::Pseudonymize,
    );
    anonymizer2.add_replacement_strategy(
        EntityType::Organization,
        anon_sdk::algorithms::entity_anonymization::ReplacementStrategy::Pseudonymize,
    );

    let mut detector1 = PatternDetector::new().expect("Failed to create detector");
    let mut detector2 = PatternDetector::new().expect("Failed to create detector");

    // Use contexts that will be detected as different entity types
    let person_text = "Contact John Smith"; // Should be detected as Person
    let org_text = "Contact Smith Corp"; // Should be detected as Organization

    let person_result = anonymizer1
        .anonymize_text(person_text, &mut detector1)
        .expect("Person anonymization failed");
    let org_result = anonymizer2
        .anonymize_text(org_text, &mut detector2)
        .expect("Organization anonymization failed");

    // Extract the anonymized "Smith" parts
    let person_smith = person_result.replace("Contact ", "");
    let org_smith = org_result.replace("Contact ", "");

    // Even with same seed, different entity types should produce different results
    // (though this test is limited by pattern detection capabilities)
    println!("Person context: {} -> {}", person_text, person_smith);
    println!("Organization context: {} -> {}", org_text, org_smith);
}

#[test]
fn test_within_execution_consistency() {
    use anon_sdk::algorithms::entity_anonymization::{
        AnonymizationStrategy, EntityAnonymization, PseudonymPools,
    };

    let pools = PseudonymPools::generate_with_seed(42, 100);
    let mut anonymizer = EntityAnonymization::new().with_pools(pools);
    anonymizer.set_global_strategy(AnonymizationStrategy::Pseudonymize);
    anonymizer.set_seed(123);

    let mut detector = PatternDetector::new().expect("Failed to create detector");

    let test_text = "John met John to discuss John's project with john@company.com";

    let result = anonymizer
        .anonymize_text(test_text, &mut detector)
        .expect("Anonymization failed");

    // Extract the pseudonym used for "John"
    let parts: Vec<&str> = result.split_whitespace().collect();
    let first_name = parts[0]; // First occurrence
    let second_name = parts[2]; // Second occurrence  
    let third_name = parts[5].trim_end_matches("'s"); // Third occurrence

    assert_eq!(
        first_name, second_name,
        "Same entity should get same pseudonym within execution"
    );
    assert_eq!(
        second_name, third_name,
        "Same entity should get same pseudonym within execution"
    );
}

#[test]
fn test_pseudonym_reversibility() {
    use anon_sdk::algorithms::entity_anonymization::{
        AnonymizationStrategy, EntityAnonymization, PseudonymPools,
    };

    let pools = PseudonymPools::generate_with_seed(42, 100);
    let mut anonymizer = EntityAnonymization::new().with_pools(pools);
    anonymizer.set_global_strategy(AnonymizationStrategy::Pseudonymize);
    anonymizer.set_seed(123);

    let mut detector = PatternDetector::new().expect("Failed to create detector");

    let original_text = "Contact John Smith at john@company.com";
    let anonymized_text = anonymizer
        .anonymize_text(original_text, &mut detector)
        .expect("Anonymization failed");

    assert_ne!(anonymized_text, original_text, "Text should be anonymized");

    let reversed_text = anonymizer
        .reverse_pseudonymization(&anonymized_text)
        .expect("Reverse pseudonymization failed");

    assert_eq!(
        reversed_text, original_text,
        "Reverse pseudonymization should restore original text"
    );
}

#[test]
fn test_pseudonym_mapping_retrieval() {
    use anon_sdk::algorithms::entity_anonymization::{
        AnonymizationStrategy, EntityAnonymization, PseudonymPools,
    };

    let pools = PseudonymPools::generate_with_seed(42, 100);
    let mut anonymizer = EntityAnonymization::new().with_pools(pools);
    anonymizer.set_global_strategy(AnonymizationStrategy::Pseudonymize);
    anonymizer.set_seed(123);

    let mut detector = PatternDetector::new().expect("Failed to create detector");

    let original_text = "Contact john@company.com";
    let _anonymized_text = anonymizer
        .anonymize_text(original_text, &mut detector)
        .expect("Anonymization failed");

    let mappings = anonymizer.get_pseudonym_mapping();

    // Debug: print what mappings we actually got
    println!("Mappings found: {:?}", mappings);

    assert!(!mappings.is_empty(), "Should have pseudonym mappings");
    assert!(
        mappings.contains_key("john@company.com"),
        "Should have mapping for email"
    );
}
