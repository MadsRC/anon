use anon_sdk::algorithms::entity_anonymization::{EntityAnonymization, ReplacementStrategy};
use anon_sdk::detection::{EntityDetector, EntityType, patterns::PatternDetector};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔒 Anonymization SDK - NER Integration Demo\n");

    // Create pattern-based detector (works without external dependencies)
    let mut detector = PatternDetector::new()?;

    // Sample text with various PII types
    let sensitive_text = r#"
    Dear John Smith,
    
    Thank you for your application. Here are your details:
    - Email: john.smith@company.com
    - Phone: (555) 123-4567  
    - SSN: 123-45-6789
    - Credit Card: 4532-1234-5678-9012
    - IP Address: 192.168.1.100
    
    Please contact our office at support@company.org or call 555-987-6543.
    
    Best regards,
    Customer Service Team
    "#;

    println!("📄 Original Text:\n{}\n", sensitive_text);

    // Detect entities
    println!("🔍 Detected Entities:");
    let entities = detector.detect(sensitive_text)?;

    for (i, entity) in entities.iter().enumerate() {
        println!(
            "  {}. {} at position {}-{}: '{}'",
            i + 1,
            entity.entity_type,
            entity.start,
            entity.end,
            entity.text
        );
    }
    println!();

    // Configure anonymization strategies
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
    anonymizer.add_replacement_strategy(EntityType::CreditCard, ReplacementStrategy::Redact);
    anonymizer.add_replacement_strategy(
        EntityType::IpAddress,
        ReplacementStrategy::Generalize("IP_ADDRESS".to_string()),
    );

    // Apply anonymization
    let anonymized_text = anonymizer.anonymize_text(sensitive_text, &mut detector)?;

    println!("🎭 Anonymized Text:\n{}\n", anonymized_text);

    // Demonstrate format-preserving anonymization
    println!("🔄 Format-Preserving Anonymization:");
    let mut format_preserving_anonymizer = EntityAnonymization::new().with_preserve_format(true);

    let test_cases = vec![
        "john.doe@example.com",
        "(555) 123-4567",
        "123-45-6789",
        "4532-1234-5678-9012",
    ];

    for test_case in test_cases {
        let anonymized = format_preserving_anonymizer.anonymize_text(test_case, &mut detector)?;
        println!("  '{}' → '{}'", test_case, anonymized);
    }

    println!("\n✅ Demo completed successfully!");
    println!("\n💡 Note: To enable advanced NER with gline-rs, use --features ner");
    println!("   This would allow detection of entities like person names and locations.");

    Ok(())
}
