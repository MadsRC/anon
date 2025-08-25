//! Integration tests for trait-based EntityAnonymization refactor
use anon_sdk::algorithms::entity_anonymization::{
    EntityAnonymization, PseudonymPools, ReplacementStrategy,
};
use anon_sdk::detection::{DetectedEntity, EntityType};
use rand::SeedableRng;

#[test]
fn test_trait_based_email_redaction() {
    // Test that EntityAnonymization uses trait-based approach for email redaction
    let mut anonymizer = EntityAnonymization::new();
    anonymizer.add_replacement_strategy(EntityType::Email, ReplacementStrategy::Redact);

    let entity = DetectedEntity::new(
        EntityType::Email,
        "user@example.com".to_string(),
        0,
        15,
        1.0,
    );

    let result = anonymizer.generate_replacement(&entity).unwrap();
    assert_eq!(result, "****@example.com");
}

#[test]
fn test_trait_based_phone_suppression() {
    // Test that EntityAnonymization uses trait-based approach for phone suppression
    let mut anonymizer = EntityAnonymization::new();
    anonymizer.add_replacement_strategy(
        EntityType::PhoneNumber,
        ReplacementStrategy::Suppress("[PHONE]".to_string()),
    );

    let entity = DetectedEntity::new(
        EntityType::PhoneNumber,
        "555-123-4567".to_string(),
        0,
        12,
        1.0,
    );

    let result = anonymizer.generate_replacement(&entity).unwrap();
    assert_eq!(result, "[PHONE]");
}

#[test]
fn test_trait_based_person_generalization() {
    // Test that EntityAnonymization uses trait-based approach for person generalization
    let mut anonymizer = EntityAnonymization::new();
    anonymizer.add_replacement_strategy(
        EntityType::Person,
        ReplacementStrategy::Generalize("PERSON".to_string()),
    );

    let entity = DetectedEntity::new(EntityType::Person, "John Doe".to_string(), 0, 8, 1.0);

    let result = anonymizer.generate_replacement(&entity).unwrap();
    assert_eq!(result, "[PERSON]");
}

#[test]
fn test_trait_based_email_pseudonymization() {
    // Test that EntityAnonymization uses trait-based approach for email pseudonymization
    let pools = PseudonymPools::new()
        .with_email_domains(vec!["test.com".to_string(), "example.org".to_string()]);

    let mut anonymizer = EntityAnonymization::new().with_pools(pools);
    anonymizer.set_seed(42);
    anonymizer.add_replacement_strategy(EntityType::Email, ReplacementStrategy::Pseudonymize);

    let entity = DetectedEntity::new(
        EntityType::Email,
        "user@example.com".to_string(),
        0,
        15,
        1.0,
    );

    let result = anonymizer.generate_replacement(&entity).unwrap();
    assert!(result.contains("@"));
    assert!(result.contains("test.com") || result.contains("example.org"));
    assert_ne!(result, "user@example.com");
}

#[test]
fn test_trait_based_person_pseudonymization() {
    // Test that EntityAnonymization uses trait-based approach for person pseudonymization
    let pools = PseudonymPools::new()
        .with_first_names(vec!["Alice".to_string(), "Bob".to_string()])
        .with_last_names(vec!["Smith".to_string(), "Jones".to_string()]);

    let mut anonymizer = EntityAnonymization::new().with_pools(pools);
    anonymizer.set_seed(42);
    anonymizer.add_replacement_strategy(EntityType::Person, ReplacementStrategy::Pseudonymize);

    let entity = DetectedEntity::new(EntityType::Person, "John Doe".to_string(), 0, 8, 1.0);

    let result = anonymizer.generate_replacement(&entity).unwrap();
    assert!(result.contains(" "));
    assert_ne!(result, "John Doe");
}

#[test]
fn test_trait_based_consistency_across_calls() {
    // Test that trait-based approach maintains consistency
    let pools = PseudonymPools::new().with_email_domains(vec!["test.com".to_string()]);

    let mut anonymizer = EntityAnonymization::new().with_pools(pools);
    anonymizer.set_seed(42);
    anonymizer.add_replacement_strategy(EntityType::Email, ReplacementStrategy::Pseudonymize);

    let entity = DetectedEntity::new(
        EntityType::Email,
        "same@example.com".to_string(),
        0,
        15,
        1.0,
    );

    let result1 = anonymizer.generate_replacement(&entity).unwrap();
    let result2 = anonymizer.generate_replacement(&entity).unwrap();
    assert_eq!(result1, result2);
}

// Tests for additional entity types that should be covered by trait-based approach

#[test]
fn test_trait_based_organization_redaction() {
    let mut anonymizer = EntityAnonymization::new();
    anonymizer.add_replacement_strategy(EntityType::Organization, ReplacementStrategy::Redact);

    let entity = DetectedEntity::new(
        EntityType::Organization,
        "Acme Corporation".to_string(),
        0,
        15,
        1.0,
    );

    let result = anonymizer.generate_replacement(&entity).unwrap();
    assert_eq!(result, "********** Corp"); // OrganizationStrategy shows min(10, len) asterisks + " Corp"
}

#[test]
fn test_trait_based_location_redaction() {
    let mut anonymizer = EntityAnonymization::new();
    anonymizer.add_replacement_strategy(EntityType::Location, ReplacementStrategy::Redact);

    let entity = DetectedEntity::new(EntityType::Location, "New York".to_string(), 0, 8, 1.0);

    let result = anonymizer.generate_replacement(&entity).unwrap();
    assert_eq!(result, "********"); // LocationStrategy redacts entire text
}

#[test]
fn test_trait_based_ssn_redaction() {
    let mut anonymizer = EntityAnonymization::new();
    anonymizer.add_replacement_strategy(
        EntityType::SocialSecurityNumber,
        ReplacementStrategy::Redact,
    );

    let entity = DetectedEntity::new(
        EntityType::SocialSecurityNumber,
        "123-45-6789".to_string(),
        0,
        11,
        1.0,
    );

    let result = anonymizer.generate_replacement(&entity).unwrap();
    assert_eq!(result, "***-**-****");
}

#[test]
fn test_trait_based_credit_card_redaction() {
    let mut anonymizer = EntityAnonymization::new();
    anonymizer.add_replacement_strategy(EntityType::CreditCard, ReplacementStrategy::Redact);

    let entity = DetectedEntity::new(
        EntityType::CreditCard,
        "4532-1234-5678-9012".to_string(),
        0,
        19,
        1.0,
    );

    let result = anonymizer.generate_replacement(&entity).unwrap();
    assert_eq!(result, "****-****-****-9012"); // CreditCardStrategy shows last 4 digits
}

#[test]
fn test_trait_based_ip_address_redaction() {
    let mut anonymizer = EntityAnonymization::new();
    anonymizer.add_replacement_strategy(EntityType::IpAddress, ReplacementStrategy::Redact);

    let entity = DetectedEntity::new(EntityType::IpAddress, "192.168.1.1".to_string(), 0, 11, 1.0);

    let result = anonymizer.generate_replacement(&entity).unwrap();
    assert_eq!(result, "***********"); // IpAddressStrategy redacts entire text
}

// Test for documentation example: Using Custom EntityType approach
#[test]
fn test_custom_entity_type_bank_account() {
    // This test demonstrates Approach 1 from the documentation
    let bank_account_type = EntityType::Custom("BankAccount".to_string());
    let mut anonymizer = EntityAnonymization::new();

    // Test redaction
    anonymizer.add_replacement_strategy(bank_account_type.clone(), ReplacementStrategy::Redact);
    let entity = DetectedEntity::new(
        bank_account_type.clone(),
        "ACC-123456789".to_string(),
        0,
        13,
        1.0,
    );
    let result = anonymizer.generate_replacement(&entity).unwrap();
    assert_eq!(result, "*************"); // Custom entity types get fully redacted

    // Test suppression
    let mut anonymizer2 = EntityAnonymization::new();
    anonymizer2.add_replacement_strategy(
        bank_account_type.clone(),
        ReplacementStrategy::Suppress("[BANK_ACCOUNT]".to_string()),
    );
    let result = anonymizer2.generate_replacement(&entity).unwrap();
    assert_eq!(result, "[BANK_ACCOUNT]");

    // Test generalization
    let mut anonymizer3 = EntityAnonymization::new();
    anonymizer3.add_replacement_strategy(
        bank_account_type.clone(),
        ReplacementStrategy::Generalize("BANK_ACCOUNT".to_string()),
    );
    let result = anonymizer3.generate_replacement(&entity).unwrap();
    assert_eq!(result, "[BANK_ACCOUNT]");

    // Test pseudonymization
    let pools = PseudonymPools::new();
    let mut anonymizer4 = EntityAnonymization::new().with_pools(pools);
    anonymizer4
        .add_replacement_strategy(bank_account_type.clone(), ReplacementStrategy::Pseudonymize);
    let result = anonymizer4.generate_replacement(&entity).unwrap();
    assert_eq!(result, "[PSEUDO_BANKACCOUNT]"); // Custom pseudonymization format
}

// Test demonstrating the trait methods directly (as shown in documentation)
#[test]
fn test_custom_entity_trait_methods() {
    use anon_sdk::algorithms::traits::EntityAnonymizationStrategy;

    let custom_entity = EntityType::Custom("BankAccount".to_string());

    // Test trait methods directly
    assert_eq!(custom_entity.redact("ACC-123456789"), "*************");
    assert_eq!(custom_entity.suppress(), "[BANKACCOUNT]");
    assert_eq!(custom_entity.generalize(), "BANKACCOUNT");

    let pools = PseudonymPools::new();
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let result = custom_entity.pseudonymize("ACC-123456789", &mut rng, &pools);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "[PSEUDO_BANKACCOUNT]");
}
