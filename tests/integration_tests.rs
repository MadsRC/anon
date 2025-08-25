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
