use crate::Result;
use crate::algorithms::entity_anonymization::PseudonymPools;
use rand::rngs::StdRng;

/// Core trait for entity-specific anonymization strategies
pub trait EntityAnonymizationStrategy {
    /// Generate format-preserving redacted version
    fn redact(&self, text: &str) -> String;

    /// Generate suppression placeholder
    fn suppress(&self) -> String;

    /// Generate generalization label
    fn generalize(&self) -> String;

    /// Generate realistic pseudonym
    fn pseudonymize(&self, text: &str, rng: &mut StdRng, pools: &PseudonymPools) -> Result<String>;
}

/// Context struct for strategy execution
pub struct StrategyContext<'a> {
    pub text: &'a str,
    pub rng: &'a mut StdRng,
    pub pools: &'a PseudonymPools,
    pub preserve_format: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithms::entity_anonymization::PseudonymPools;
    use crate::detection::EntityType;
    use rand::{SeedableRng, rngs::StdRng};

    #[test]
    fn test_entity_type_redact_email() {
        let entity_type = EntityType::Email;
        let result = entity_type.redact("user@example.com");
        assert_eq!(result, "****@example.com");
    }

    #[test]
    fn test_entity_type_suppress_email() {
        let entity_type = EntityType::Email;
        let result = entity_type.suppress();
        assert_eq!(result, "[EMAIL]");
    }

    #[test]
    fn test_entity_type_generalize_email() {
        let entity_type = EntityType::Email;
        let result = entity_type.generalize();
        assert_eq!(result, "EMAIL");
    }

    #[test]
    fn test_entity_type_pseudonymize_email() {
        let entity_type = EntityType::Email;
        let pools = PseudonymPools::new()
            .with_email_domains(vec!["test.com".to_string(), "example.org".to_string()]);
        let mut rng = StdRng::seed_from_u64(42);

        let result = entity_type.pseudonymize("user@example.com", &mut rng, &pools);
        assert!(result.is_ok());
        let pseudonym = result.unwrap();
        assert!(pseudonym.contains("@"));
        assert!(pseudonym.contains("test.com") || pseudonym.contains("example.org"));
    }

    #[test]
    fn test_entity_type_redact_phone() {
        let entity_type = EntityType::PhoneNumber;
        let result = entity_type.redact("555-123-4567");
        assert_eq!(result, "***-***-****");
    }

    #[test]
    fn test_entity_type_suppress_phone() {
        let entity_type = EntityType::PhoneNumber;
        let result = entity_type.suppress();
        assert_eq!(result, "[PHONE]");
    }

    #[test]
    fn test_entity_type_generalize_phone() {
        let entity_type = EntityType::PhoneNumber;
        let result = entity_type.generalize();
        assert_eq!(result, "PHONE");
    }

    #[test]
    fn test_entity_type_pseudonymize_phone() {
        let entity_type = EntityType::PhoneNumber;
        let pools = PseudonymPools::new(); // Phone doesn't need pools
        let mut rng = StdRng::seed_from_u64(42);

        let result = entity_type.pseudonymize("555-123-4567", &mut rng, &pools);
        assert!(result.is_ok());
        let pseudonym = result.unwrap();
        assert!(pseudonym.starts_with("555-"));
        assert_ne!(pseudonym, "555-123-4567");
    }

    #[test]
    fn test_entity_type_redact_person() {
        let entity_type = EntityType::Person;
        let result = entity_type.redact("John Doe");
        assert_eq!(result, "**** ***");
    }

    #[test]
    fn test_entity_type_suppress_person() {
        let entity_type = EntityType::Person;
        let result = entity_type.suppress();
        assert_eq!(result, "[PERSON]");
    }

    #[test]
    fn test_entity_type_generalize_person() {
        let entity_type = EntityType::Person;
        let result = entity_type.generalize();
        assert_eq!(result, "PERSON");
    }

    #[test]
    fn test_entity_type_pseudonymize_person() {
        let entity_type = EntityType::Person;
        let pools = PseudonymPools::new()
            .with_first_names(vec!["Alice".to_string(), "Bob".to_string()])
            .with_last_names(vec!["Smith".to_string(), "Jones".to_string()]);
        let mut rng = StdRng::seed_from_u64(42);

        let result = entity_type.pseudonymize("John Doe", &mut rng, &pools);
        assert!(result.is_ok());
        let pseudonym = result.unwrap();
        assert!(pseudonym.contains(" ")); // Should have first and last name
        assert_ne!(pseudonym, "John Doe");
    }
}
