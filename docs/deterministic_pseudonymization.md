# Deterministic Pseudonymization

This document explains how the anonymization SDK implements deterministic pseudonymization to ensure consistency and enable reversibility.

## Overview

The SDK provides two modes of deterministic behavior:

1. **Text-based determinism** (always active)
2. **Seed-based determinism** (when `--seed` is provided)

## How Determinism Works

### Hash-Based Generation

Pseudonyms are generated using a deterministic approach based on cryptographic hashing:

```rust
// Create hash from multiple inputs
let mut hasher = DefaultHasher::new();

// Optional: Include user-provided seed
if let Some(seed) = self.seed {
    seed.hash(&mut hasher);
}

// Always include original text and entity type
original_text.hash(&mut hasher);        // e.g., "John Smith"
entity.entity_type.hash(&mut hasher);   // e.g., EntityType::Person

// Use hash as RNG seed for consistent generation
let deterministic_seed = hasher.finish();
let mut rng = StdRng::seed_from_u64(deterministic_seed);
```

### Behavior Without Explicit Seed

**⚠️ Important**: Even without providing a `--seed` parameter, pseudonymization is still deterministic.

- `"John Smith"` will **always** become the same pseudonym (e.g., "Alex Miller")
- `"Jane Doe"` will **always** become a different but consistent pseudonym (e.g., "Taylor Davis")
- This is because the hash is computed from the original text itself

**Example:**
```bash
# Both calls produce identical output
echo "Contact John Smith" | anon
# → "Contact Alex Miller"

echo "Contact John Smith" | anon  
# → "Contact Alex Miller" (same result)
```

### Behavior With Explicit Seed

When `--seed` is provided, the pseudonyms change but remain deterministic:

```bash
echo "Contact John Smith" | anon --seed 42
# → "Contact Taylor Wilson"

echo "Contact John Smith" | anon --seed 42
# → "Contact Taylor Wilson" (same result)

echo "Contact John Smith" | anon --seed 99
# → "Contact Morgan Brown" (different but consistent)
```

## Consistency Guarantees

### Within Single Execution

✅ **Guaranteed**: Same entity → same pseudonym within one CLI call

```bash
echo "John met John to discuss John's project" | anon --seed 42
# → "Taylor met Taylor to discuss Taylor's project"
```

### Across Multiple Executions

✅ **Guaranteed**: Same input + same seed → identical output across separate CLI calls

```bash
# Call 1
echo "Contact John at john@company.com" | anon --seed 42
# → "Contact Taylor at user7007@test.org"

# Call 2 (separate process)
echo "Contact John at john@company.com" | anon --seed 42  
# → "Contact Taylor at user7007@test.org" (identical)
```

## Mapping Storage

### In-Memory Only

- Mappings are stored in RAM during execution
- **Not persisted** to disk between CLI invocations
- Consistency across calls comes from deterministic generation, not stored mappings

### Storage Structure

```rust
// Forward mapping: original → pseudonym
pseudonym_mappings: HashMap<String, String>
// "John Smith" → "Alex Miller"

// Reverse mapping: pseudonym → original  
reverse_mappings: HashMap<String, String>
// "Alex Miller" → "John Smith"
```

## Reversibility

### Getting Mappings

```rust
let mappings = anonymizer.get_pseudonym_mapping();
// Returns HashMap of all original → pseudonym mappings
```

### Reversing Anonymization

```rust
let original_text = anonymizer.reverse_pseudonymization(&anonymized_text)?;
// Restores pseudonyms back to original values
```

### CLI Usage for Reversibility

Currently, reversibility is only available programmatically. The CLI doesn't persist or load mappings between invocations.

## Use Cases

### Enterprise Workflows

- **Document Processing**: Same entities get consistent pseudonyms across multiple documents
- **Database Anonymization**: Referential integrity maintained through deterministic mapping
- **Testing**: Reproducible anonymization for test data generation

### LLM Context Preservation

- **Multi-turn Conversations**: "John Smith" in message 1 remains "Alex Miller" in message 2
- **Document Series**: Consistent character mapping across related documents
- **Audit Trails**: Ability to trace back anonymized entities when needed

## Security Considerations

### Determinism as Feature vs Bug

**Feature Perspective:**
- Ensures consistency across workflows
- Enables reversibility with proper key management
- Maintains relational data integrity

**Security Perspective:**
- Same plaintext always produces same pseudonym (potential pattern analysis)
- No built-in key derivation (users must manage seeds securely)
- Deterministic behavior without seed may leak information

### Recommendations

1. **Always use seeds** for production anonymization
2. **Rotate seeds** periodically for forward security
3. **Secure seed storage** using proper key management
4. **Consider salting** original text with additional entropy

## Implementation Notes

### Entity Type Separation

Different entity types with same text get different pseudonyms:

```bash
# If "Smith" appears as both person surname and organization
echo "John Smith works at Smith Corp" | anon --seed 42
# → "Taylor Wilson works at Alpha Dynamics"
#    ^person         ^organization (different pseudonyms)
```

### Current Limitations

- **Fixed pools**: Names selected from hardcoded lists (~10 options per category)
- **Collision potential**: Limited pool size may cause collisions with large datasets
- **No persistence**: Mappings lost between CLI invocations
- **No key derivation**: Seeds used directly without additional security layers

## Future Considerations

- Larger pseudonym pools or algorithmic generation
- Persistent mapping storage with encryption
- Key derivation functions for enhanced security
- Configurable determinism levels (strict/relaxed)