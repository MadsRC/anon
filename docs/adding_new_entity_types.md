# Adding New EntityTypes to the Anonymization System

This guide explains how to add new `EntityType` variants to the trait-based anonymization system implemented in this codebase.

## Overview

The anonymization system uses a trait-based approach where each `EntityType` implements the `EntityAnonymizationStrategy` trait. This allows for extensible and maintainable entity-specific anonymization logic.

## Two Approaches for Adding New Entity Types

### Approach 1: Using Custom EntityType (Recommended for Quick Prototyping)

For quick prototyping or external entity types, you can use the existing `EntityType::Custom(String)` variant:

```rust
use anon_sdk::detection::EntityType;
use anon_sdk::algorithms::entity_anonymization::{EntityAnonymization, ReplacementStrategy};

// Create a custom entity type
let bank_account_type = EntityType::Custom("BankAccount".to_string());

// Use it with the anonymization system
let mut anonymizer = EntityAnonymization::new();
anonymizer.add_replacement_strategy(bank_account_type.clone(), ReplacementStrategy::Redact);

// The custom entity type will use default behaviors:
// - redact(): Full asterisk replacement
// - suppress(): "[CUSTOM_NAME]" format  
// - generalize(): The custom name in uppercase
// - pseudonymize(): "[PSEUDO_CUSTOM_NAME]" format
```

### Approach 2: Adding a New EntityType Variant (For Core Integration)

For entity types that should be part of the core system, follow these steps:

#### Step 1: Add the EntityType Variant

Update `src/detection/mod.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EntityType {
    Person,
    Location,
    Organization,
    Email,
    PhoneNumber,
    SocialSecurityNumber,
    CreditCard,
    IpAddress,
    BankAccount,  // <- Add your new variant here
    Custom(String),
}
```

Don't forget to update the `Display` implementation:

```rust
impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // ... existing variants ...
            EntityType::BankAccount => write!(f, "bank_account"),
            EntityType::Custom(name) => write!(f, "custom_{}", name.to_lowercase()),
        }
    }
}
```

#### Step 2: Create the Strategy Struct

Create `src/algorithms/strategies/bank_account.rs`:

```rust
use crate::algorithms::entity_anonymization::PseudonymPools;
use crate::Result;
use rand::Rng;
use rand::rngs::StdRng;

pub struct BankAccountStrategy;

impl BankAccountStrategy {
    /// Generate format-preserving redacted version
    pub fn redact(text: &str) -> String {
        // Preserve account number structure: ACC-XXXXXXXXX
        if let Some(dash_pos) = text.find('-') {
            let prefix = &text[..dash_pos + 1];
            let suffix_len = text.len() - dash_pos - 1;
            format!("{}{}", prefix, "*".repeat(suffix_len))
        } else {
            // Fallback: full redaction
            "*".repeat(text.len())
        }
    }

    /// Generate suppression placeholder
    pub fn suppress() -> String {
        "[BANK_ACCOUNT]".to_string()
    }

    /// Generate generalization label
    pub fn generalize() -> String {
        "BANK_ACCOUNT".to_string()
    }

    /// Generate realistic pseudonym
    pub fn pseudonymize(_text: &str, rng: &mut StdRng, _pools: &PseudonymPools) -> Result<String> {
        // Bank accounts typically don't need pools - generate algorithmically
        Ok(format!("ACC-{:09}", rng.gen_range(100000000..999999999)))
    }
}
```

#### Step 3: Update the Strategy Module

Add to `src/algorithms/strategies/mod.rs`:

```rust
pub mod bank_account;  // <- Add this line

// Re-export strategy structs
pub use bank_account::BankAccountStrategy;  // <- Add this line
```

#### Step 4: Update the EntityType Trait Implementation

Add the match arms in `src/algorithms/entity_strategies.rs`:

```rust
impl EntityAnonymizationStrategy for EntityType {
    fn redact(&self, text: &str) -> String {
        match self {
            // ... existing variants ...
            EntityType::BankAccount => BankAccountStrategy::redact(text),
            EntityType::Custom(_) => "*".repeat(text.len()),
        }
    }

    fn suppress(&self) -> String {
        match self {
            // ... existing variants ...
            EntityType::BankAccount => BankAccountStrategy::suppress(),
            EntityType::Custom(name) => format!("[{}]", name.to_uppercase()),
        }
    }

    fn generalize(&self) -> String {
        match self {
            // ... existing variants ...
            EntityType::BankAccount => BankAccountStrategy::generalize(),
            EntityType::Custom(name) => name.to_uppercase(),
        }
    }

    fn pseudonymize(&self, text: &str, rng: &mut StdRng, pools: &PseudonymPools) -> Result<String> {
        match self {
            // ... existing variants ...
            EntityType::BankAccount => BankAccountStrategy::pseudonymize(text, rng, pools),
            EntityType::Custom(name) => Ok(format!("[PSEUDO_{}]", name.to_uppercase())),
        }
    }
}
```

#### Step 5: Add Comprehensive Tests

Create tests in your test file:

```rust
#[test]
fn test_bank_account_redaction() {
    let entity_type = EntityType::BankAccount;
    let result = entity_type.redact("ACC-123456789");
    assert_eq!(result, "ACC-*********");
}

#[test]
fn test_bank_account_suppression() {
    let entity_type = EntityType::BankAccount;
    let result = entity_type.suppress();
    assert_eq!(result, "[BANK_ACCOUNT]");
}

#[test]
fn test_bank_account_generalization() {
    let entity_type = EntityType::BankAccount;
    let result = entity_type.generalize();
    assert_eq!(result, "BANK_ACCOUNT");
}

#[test]
fn test_bank_account_pseudonymization() {
    let entity_type = EntityType::BankAccount;
    let pools = PseudonymPools::new();
    let mut rng = StdRng::seed_from_u64(42);

    let result = entity_type.pseudonymize("ACC-123456789", &mut rng, &pools);
    assert!(result.is_ok());
    let pseudonym = result.unwrap();
    assert!(pseudonym.starts_with("ACC-"));
    assert_eq!(pseudonym.len(), 13); // ACC- + 9 digits
}
```

## Strategy Implementation Guidelines

### Redaction Strategy Guidelines

- **Format Preservation**: Try to preserve the format structure when possible
- **Security**: Ensure no original data leaks through
- **Consistency**: Use consistent masking characters (typically "*")

### Suppression Strategy Guidelines

- **Descriptive**: Use clear, descriptive placeholders like `[BANK_ACCOUNT]`
- **Consistent**: Follow the `[ENTITY_NAME]` pattern
- **Uppercase**: Keep entity names in uppercase for consistency

### Generalization Strategy Guidelines

- **Simple Labels**: Use simple, categorical labels like `BANK_ACCOUNT`
- **Uppercase**: Maintain uppercase convention
- **No Brackets**: Unlike suppression, generalization doesn't use brackets

### Pseudonymization Strategy Guidelines

- **Realistic**: Generate realistic-looking replacements when possible
- **Deterministic**: Use the provided RNG for deterministic results
- **Format Preservation**: Maintain similar format and length when feasible
- **Pool Usage**: Use `PseudonymPools` when entity types benefit from curated lists

## Testing Your Implementation

1. **Run Individual Tests**:
   ```bash
   cargo test test_bank_account
   ```

2. **Run Full Test Suite**:
   ```bash
   cargo test
   ```

3. **Format and Lint**:
   ```bash
   cargo fmt
   cargo clippy
   ```

## Integration with Detection

If you're also adding detection capabilities for your new entity type, you'll need to:

1. **Update Pattern Detection**: Add patterns in `src/detection/patterns.rs`
2. **Update NER Models**: Configure entity recognition models if using ML-based detection
3. **Update Hybrid Detection**: Ensure your new entity type works with the hybrid detector

## Benefits of This Approach

- **Extensibility**: Easy to add new entity types without modifying existing code
- **Maintainability**: Each entity type has its own focused implementation
- **Testability**: Individual entity behaviors can be tested in isolation
- **Type Safety**: Rust compiler ensures all trait methods are implemented
- **Performance**: No runtime overhead compared to alternative approaches

## Example: Complete BankAccount Integration

See the test file `tests/trait_based_integration_tests.rs` for a working example of how custom entity types integrate with the anonymization system.

The trait-based architecture makes extending the system straightforward while maintaining code quality and ensuring comprehensive test coverage.