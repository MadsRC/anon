# EntityType Anonymization Refactor: Trait-Based Implementation Plan

## Overview
Refactor the current match-statement based anonymization system to use a trait-based approach for better extensibility, maintainability, and adherence to Rust idioms.

## Goals
- **Open/Closed Principle**: Add new EntityTypes without modifying existing code
- **Single Responsibility**: Each entity handles its own anonymization logic
- **Compile-Time Safety**: Ensure all strategies are implemented for each entity
- **Testability**: Enable isolated testing of individual entity behaviors
- **Maintainability**: Reduce code duplication and centralized logic

## Implementation Plan

### Phase 1: Define Core Traits and Types

#### 1.1 Create `AnonymizationStrategy` Trait
**File**: `src/algorithms/traits.rs` (new)
```rust
pub trait AnonymizationStrategy {
    /// Generate format-preserving redacted version
    fn redact(&self, text: &str) -> String;
    
    /// Generate suppression placeholder
    fn suppress(&self) -> String;
    
    /// Generate generalization label
    fn generalize(&self) -> String;
    
    /// Generate realistic pseudonym
    fn pseudonymize(&self, text: &str, rng: &mut StdRng, pools: &PseudonymPools) -> Result<String>;
}
```

#### 1.2 Create Strategy Context Struct
```rust
pub struct StrategyContext<'a> {
    pub text: &'a str,
    pub rng: &'a mut StdRng,
    pub pools: &'a PseudonymPools,
    pub preserve_format: bool,
}
```

### Phase 2: Implement Trait for EntityType

#### 2.1 Core Implementation
**File**: `src/algorithms/entity_strategies.rs` (new)
```rust
impl AnonymizationStrategy for EntityType {
    fn redact(&self, text: &str) -> String {
        match self {
            EntityType::Email => EmailStrategy::redact(text),
            EntityType::PhoneNumber => PhoneStrategy::redact(text),
            EntityType::Person => PersonStrategy::redact(text),
            // ... delegate to specific strategy structs
        }
    }
    
    fn suppress(&self) -> String {
        match self {
            EntityType::Email => EmailStrategy::suppress(),
            EntityType::PhoneNumber => PhoneStrategy::suppress(),
            // ...
        }
    }
    
    // Similar for generalize() and pseudonymize()
}
```

### Phase 3: Create Individual Strategy Structs

#### 3.1 Strategy Struct Pattern
**File**: `src/algorithms/strategies/mod.rs` (new directory)
```rust
// Individual strategy modules
pub mod email;
pub mod phone;
pub mod person;
pub mod ip_address;
// ...

// Re-export strategy structs
pub use email::EmailStrategy;
pub use phone::PhoneStrategy;
// ...
```

#### 3.2 Example Individual Strategy
**File**: `src/algorithms/strategies/email.rs`
```rust
pub struct EmailStrategy;

impl EmailStrategy {
    pub fn redact(text: &str) -> String {
        if let Some(at_pos) = text.find('@') {
            let (local_part, domain_part) = text.split_at(at_pos);
            format!("{}@{}", "*".repeat(local_part.len()), &domain_part[1..])
        } else {
            "*".repeat(text.len())
        }
    }
    
    pub fn suppress() -> String {
        "[EMAIL]".to_string()
    }
    
    pub fn generalize() -> String {
        "EMAIL".to_string()
    }
    
    pub fn pseudonymize(
        _text: &str, 
        rng: &mut StdRng, 
        pools: &PseudonymPools
    ) -> Result<String> {
        let domains = pools.email_domains.as_ref()
            .ok_or_else(|| AnonError::InvalidInput("No email domains pool".to_string()))?;
        
        Ok(format!(
            "user{}@{}",
            rng.gen_range(1000..9999),
            domains[rng.gen_range(0..domains.len())]
        ))
    }
}
```

### Phase 4: Update EntityAnonymization Struct

#### 4.1 Simplified Main Logic
```rust
impl EntityAnonymization {
    pub fn anonymize_text(&mut self, text: &str, detector: &mut dyn EntityDetector) -> Result<String> {
        let mut all_entities = detector.detect(text)?;
        // ... existing entity processing logic ...
        
        for entity in &all_entities {
            let replacement = self.generate_replacement_trait_based(entity)?;
            // ... existing replacement logic ...
        }
        
        Ok(result)
    }
    
    fn generate_replacement_trait_based(&mut self, entity: &DetectedEntity) -> Result<String> {
        let strategy = self.get_strategy_for_entity(entity);
        
        match strategy {
            ReplacementStrategy::Redact => {
                Ok(entity.entity_type.redact(&entity.text))
            }
            ReplacementStrategy::Suppress(_) => {
                Ok(entity.entity_type.suppress())
            }
            ReplacementStrategy::Generalize(_) => {
                Ok(format!("[{}]", entity.entity_type.generalize()))
            }
            ReplacementStrategy::Pseudonymize => {
                let pools = self.get_pools()?;
                let mut rng = self.create_deterministic_rng(&entity.text, &entity.entity_type);
                entity.entity_type.pseudonymize(&entity.text, &mut rng, pools)
            }
        }
    }
}
```

### Phase 5: Migration Strategy

#### 5.1 Backward Compatibility
- Keep existing `generate_replacement()` method during transition
- Add feature flag to switch between old and new implementations
- Gradually migrate tests to use trait-based approach

#### 5.2 Testing Strategy
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_email_redaction() {
        let result = EmailStrategy::redact("user@example.com");
        assert_eq!(result, "****@example.com");
    }
    
    #[test]
    fn test_entity_type_delegation() {
        let result = EntityType::Email.redact("user@example.com");
        assert_eq!(result, "****@example.com");
    }
}
```

### Phase 6: Extension Points for New EntityTypes

#### 6.1 Adding New EntityType Process
1. **Add to enum**: Update `EntityType` in `detection/mod.rs`
2. **Create strategy struct**: Add new file in `algorithms/strategies/`
3. **Update delegation**: Add match arm in `entity_strategies.rs`
4. **Add tests**: Create comprehensive test suite

#### 6.2 Example New EntityType Addition
```rust
// 1. In detection/mod.rs
pub enum EntityType {
    // ... existing variants
    BankAccount,  // New entity
}

// 2. In algorithms/strategies/bank_account.rs
pub struct BankAccountStrategy;
impl BankAccountStrategy {
    pub fn redact(text: &str) -> String { /* implementation */ }
    pub fn suppress() -> String { "[BANK_ACCOUNT]".to_string() }
    pub fn generalize() -> String { "BANK_ACCOUNT".to_string() }
    pub fn pseudonymize(text: &str, rng: &mut StdRng, pools: &PseudonymPools) -> Result<String> {
        Ok(format!("ACC{:08}", rng.gen_range(10000000..99999999)))
    }
}

// 3. In entity_strategies.rs - add to match statements
EntityType::BankAccount => BankAccountStrategy::redact(text),
```

## Benefits of This Approach

1. **Extensibility**: New EntityTypes only require adding their strategy struct
2. **Testability**: Each entity's behavior can be tested in isolation
3. **Maintainability**: Logic is distributed and organized by entity type
4. **Performance**: No runtime overhead compared to current approach
5. **Type Safety**: Compiler ensures all strategies are implemented
6. **Documentation**: Each strategy struct serves as clear documentation

This refactor will significantly improve the codebase's maintainability and make it much easier to add new EntityTypes in the future.