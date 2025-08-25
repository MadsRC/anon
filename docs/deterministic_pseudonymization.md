# Deterministic Pseudonymization

This document explains how the anonymization SDK implements deterministic pseudonymization to ensure consistency and enable reversibility.

## Overview

The SDK provides deterministic pseudonymization through multiple mechanisms:

1. **Hash-based determinism** (always active) - Same text + entity type produces same pseudonym
2. **Persistent seed management** (CLI automatic) - Ensures consistency across CLI invocations  
3. **Explicit seed control** (when `--seed` is provided) - User-controlled determinism

## How Determinism Works

### Hash-Based Generation

Pseudonyms are generated using a deterministic approach based on cryptographic hashing:

```rust
// Create hash from multiple inputs
let mut hasher = std::collections::hash_map::DefaultHasher::new();
use std::hash::{Hash, Hasher};

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

**⚠️ Important**: Even without providing a `--seed` parameter, pseudonymization is still deterministic through persistent seed management.

- When no `--seed` is provided, the CLI automatically generates and persists a seed
- `"John Smith"` will **always** become the same pseudonym (e.g., "Alex Miller") within the same persistent session
- `"Jane Doe"` will **always** become a different but consistent pseudonym (e.g., "Taylor Davis")
- This is because the hash is computed from the persistent seed + original text + entity type

**Example:**
```bash
# First call generates and persists seed automatically
echo "Contact John Smith" | anon
# → "Contact Alex Miller" (persistent seed: 12345678901234567890)

# Second call uses same persistent seed
echo "Contact John Smith" | anon  
# → "Contact Alex Miller" (same result with same persistent seed)
```

### Persistent Seed Management

The CLI automatically manages seeds to ensure consistency across invocations:

- **Persistent seed file**: Stored in `~/.anon/seed` 
- **Automatic generation**: Created on first use if no explicit seed provided
- **Explicit override**: `--seed` parameter overrides persistent seed for that invocation
- **Consistency guarantee**: Same operations produce identical results across separate CLI calls

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

Reversibility is available programmatically through the SDK. While the CLI doesn't persist pseudonym mappings between invocations, it does maintain persistent seeds to ensure consistency across separate CLI calls, enabling deterministic re-generation of the same pseudonyms.

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

Different entity types with same text get different pseudonyms due to entity type being included in the hash:

```bash
# If "Smith" appears as both person surname and organization
echo "John Smith works at Smith Corp" | anon --seed 42
# → "Taylor Wilson works at Alpha Dynamics"
#    ^person         ^organization (different pseudonyms)
```

This separation is achieved by including the `entity.entity_type` in the deterministic hash calculation, ensuring the same text produces different pseudonyms when detected as different entity types.

### Pseudonym Pool Generation

The system generates realistic pseudonym pools using algorithmic methods:

```rust
// Pools are generated with configurable size (default: 10,000)
let pools = PseudonymPools::generate_with_seed(seed, pool_size);

// Generated pools include:
// - first_names: Algorithmically generated realistic first names
// - last_names: Algorithmically generated realistic last names  
// - organizations: Generated company names with prefixes/suffixes
// - locations: Generated city/place names
// - email_domains: Generated domains for email pseudonyms
```

**Pool Management Commands:**
- `anon generate-pools --size 10000 --seed 42` - Generate new pools
- `anon show-pools` - Display pool information and metadata
- `anon export-pools output.json` - Export current pools
- `anon import-pools input.json --set-default` - Import external pools

### Current Limitations

- **Configurable pools**: Names selected from generated pools (default 10,000 pseudonyms per category, configurable)
- **Collision potential**: With large datasets, pool exhaustion may cause repeated pseudonyms
- **No mapping persistence**: Pseudonym mappings lost between CLI invocations (but seeds are persisted for consistency)
- **Direct seed usage**: Seeds used directly in hash function without additional key derivation layers

## Future Considerations

- Even larger pseudonym pools or enhanced algorithmic generation
- Persistent mapping storage with encryption
- Key derivation functions for enhanced security
- Configurable determinism levels (strict/relaxed)
- Optional mapping persistence across CLI invocations