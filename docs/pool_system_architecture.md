# Pool System Architecture

The anonymization SDK uses a **pool-based pseudonymization system** where pseudonyms are selected from pre-generated pools rather than hardcoded arrays. This design enhances security, scalability, and team collaboration.

## Overview

### Security Model

**Enhanced Multi-Factor Protection:**
- **Factor 1**: Seed (controls selection from pools)
- **Factor 2**: Pools (define universe of possible pseudonyms)  
- **Factor 3**: Algorithm (deterministic selection logic)

**Attack Resistance:**
- **Seed compromised**: Attacker still can't reverse without pools
- **Pools compromised**: Attacker still can't reverse without seed  
- **Both required**: True cryptographic security

### Architecture Layers

```
┌─────────────────────────────────────────┐
│                   CLI                   │ ← Pool Management
├─────────────────────────────────────────┤
│              Library Core               │ ← Pool Consumption  
├─────────────────────────────────────────┤
│            Detection Engine             │ ← Entity Detection
└─────────────────────────────────────────┘
```

## Pool Structure

### PseudonymPools Format

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PseudonymPools {
    pub first_names: Option<Vec<String>>,      // 1K-10K names
    pub last_names: Option<Vec<String>>,       // 1K-10K surnames
    pub organizations: Option<Vec<String>>,    // 1K-10K companies
    pub locations: Option<Vec<String>>,        // 1K-10K places
    pub email_domains: Option<Vec<String>>,    // 20-100 domains
}
```

### Pool Generation Strategy

**Algorithmic Generation** (infinite scalability):
```rust
PseudonymPools::generate_with_seed(seed, size)
```

**Benefits:**
- ✅ **Deterministic**: Same seed + size = identical pools
- ✅ **Scalable**: Generate 1K, 10K, 100K+ pseudonyms  
- ✅ **Realistic**: Syllable-based name construction
- ✅ **Collision-resistant**: Massive pool sizes prevent overlap

### Sample Generated Names

**Seed 12345, Size 10:**
```json
{
  "first_names": ["Alor", "Jolan", "Taex", "Caey", "Rian"],
  "last_names": ["Smith", "Jolliams", "Wilown", "Bravis", "Danis"],
  "organizations": ["Tech Corp", "Data Systems", "Global Solutions"],
  "locations": ["Springfield", "Riverside", "Franklin"]
}
```

## CLI Workflow Design

### Default Pool Management

```bash
# First run - auto-generates pools
$ anon input.txt
⚠️  No pools found. Generating default pools (10,000 pseudonyms)...
✅ Generated pools cached to ~/.anon/pools/default_pools.json
📄 Processing input.txt...

# Subsequent runs - uses cached pools
$ anon input2.txt  
✅ Using cached pools from ~/.anon/pools/default_pools.json
📄 Processing input2.txt...
```

### Pool Generation Commands

```bash
# Generate custom pool size
$ anon --generate-pools --pool-size 50000
✅ Generated 50,000 pseudonyms → ~/.anon/pools/default_pools.json

# Generate with specific seed (team coordination)
$ anon --generate-pools --pool-seed 12345 --pool-size 10000
✅ Generated pools with seed 12345 → ~/.anon/pools/default_pools.json

# Export for sharing
$ anon --export-pools team_pools.json
✅ Exported current pools → team_pools.json
```

### Pool Sharing Commands  

```bash
# Use specific pool file
$ anon input.txt --pools team_pools.json
✅ Using pools from team_pools.json

# Import and cache pools
$ anon --import-pools team_pools.json  
✅ Imported team_pools.json → ~/.anon/pools/default_pools.json

# Generate pools for sharing
$ anon --generate-pools --pool-seed $SHARED_SECRET --export-pools project_pools.json
✅ Generated deterministic pools → project_pools.json
```

## File System Layout

### Default Pool Location
```
~/.anon/
├── pools/
│   ├── default_pools.json      ← Auto-generated default pools
│   ├── backup_pools.json       ← Automatic backup of previous
│   └── project_team_pools.json ← Manually imported pools
└── config.json                 ← CLI configuration
```

### Pool File Format

```json
{
  "metadata": {
    "version": "1.0",
    "generated_at": "2024-01-15T10:30:00Z", 
    "pool_seed": 12345,
    "pool_size": 10000,
    "generator_version": "0.1.0"
  },
  "pools": {
    "first_names": ["Alor", "Jolan", "Taex", ...],
    "last_names": ["Smith", "Jolliams", "Wilown", ...], 
    "organizations": ["Tech Corp", "Data Systems", ...],
    "locations": ["Springfield", "Riverside", ...],
    "email_domains": ["example001.com", "test002.org", ...]
  }
}
```

## Team Collaboration Workflows

### Scenario 1: Centralized Pool Distribution

```bash
# Data Controller (generates and distributes)
$ anon --generate-pools --pool-seed $COMPANY_SECRET --export-pools company_pools.json
$ # Share company_pools.json via secure channel

# Team Members (import shared pools)  
$ anon --import-pools company_pools.json
$ anon dataset1.csv --seed $PROJECT_SEED > anonymized1.csv
$ anon dataset2.csv --seed $PROJECT_SEED > anonymized2.csv
# Result: Consistent pseudonyms across all team members
```

### Scenario 2: Distributed Pool Generation

```bash
# All team members generate identical pools
$ anon --generate-pools --pool-seed 12345 --pool-size 10000
$ anon data.csv --seed 67890 > output.csv
# Result: Same pools + same seed = identical output across systems
```

### Scenario 3: Project-Specific Pools

```bash
# Project Lead
$ anon --generate-pools --pool-seed $PROJECT_ID --export-pools project_alpha_pools.json

# Analysts  
$ anon dataset.csv --pools project_alpha_pools.json --seed $DATA_VERSION
# Result: Project-isolated pseudonyms, no cross-project correlation
```

## Security Considerations

### Pool as Key Material

**Critical Understanding**: Pools are cryptographic key material, not just configuration.

- **Confidentiality**: Pools must be secured like encryption keys
- **Integrity**: Pool tampering changes pseudonym mappings
- **Versioning**: Pool changes break deterministic reproduction

### Threat Model

**Attacks Mitigated:**
1. **Seed-only compromise**: Pools remain secret → partial protection
2. **Pool-only compromise**: Seed remains secret → partial protection  
3. **Pattern analysis**: Large pools prevent frequency analysis
4. **Collision attacks**: Algorithmic generation scales to prevent collisions

**Attacks NOT Mitigated:**
1. **Full compromise**: Both pools + seed compromised → complete break
2. **Side-channel**: Pool generation timing/memory patterns
3. **Implementation bugs**: RNG weaknesses, hash collisions

### Best Practices

1. **Secure Pool Storage**: Encrypt pool files at rest
2. **Access Control**: Limit pool file permissions (600)
3. **Key Rotation**: Regenerate pools periodically  
4. **Audit Trails**: Log pool generation/usage events
5. **Backup Strategy**: Secure backup of pools for data recovery

## Determinism Guarantees

### Reproducibility Requirements

For identical output across systems/time:

1. **Same Pool Seed**: `--pool-seed` must match
2. **Same Pool Size**: `--pool-size` must match  
3. **Same Data Seed**: `--seed` must match
4. **Same Input**: Text content must match
5. **Same Version**: Library version should match

### Determinism Levels

**Level 1 - Text Determinism** (always active):
- Same text + same pools + same seed = same pseudonym

**Level 2 - Cross-System Determinism** (with shared pools):
- Same inputs across different machines = identical outputs

**Level 3 - Cross-Time Determinism** (with version control):
- Same inputs months later = identical outputs (if pools preserved)

## Migration Guide

### From Hardcoded to Pool-Based

**Before (v0.x)**:
```rust
let anonymizer = EntityAnonymization::new();
// Worked with hardcoded 100-name pools
```

**After (v1.x)**:
```rust
let pools = PseudonymPools::generate_with_seed(12345, 10000);
let anonymizer = EntityAnonymization::new().with_pools(pools);
// Now requires explicit pools - breaking change
```

### CLI Migration

**Before**: `anon input.txt` (worked immediately)  
**After**: `anon input.txt` (auto-generates pools on first run)

**Result**: Seamless migration with improved security

## Performance Characteristics

### Pool Generation Performance

| Pool Size | Generation Time | Memory Usage | Disk Size |
|-----------|----------------|---------------|-----------|
| 1,000     | ~1ms           | ~50KB        | ~25KB     |
| 10,000    | ~10ms          | ~500KB       | ~250KB    |
| 100,000   | ~100ms         | ~5MB         | ~2.5MB    |
| 1,000,000 | ~1s            | ~50MB        | ~25MB     |

### Runtime Performance Impact

- **Pool Loading**: One-time JSON parsing (~1-10ms)
- **Pseudonym Generation**: No performance impact (same algorithm)
- **Memory Overhead**: Proportional to pool size (negligible for <100K pools)

## Future Enhancements

### Planned Features

1. **Pool Encryption**: Encrypt pools at rest with master key
2. **Pool Compression**: Reduce storage overhead for large pools  
3. **Pool Versioning**: Support multiple pool versions simultaneously
4. **Pool Validation**: Cryptographic signatures for pool integrity
5. **Pool Templates**: Industry-specific name patterns (medical, legal, etc.)

### Advanced Use Cases

1. **Hierarchical Pools**: Department-specific sub-pools
2. **Time-Rotating Pools**: Automatic pool regeneration schedules
3. **Geographic Pools**: Region-appropriate names/organizations
4. **Compliance Pools**: GDPR/HIPAA-compliant pseudonym patterns

## Troubleshooting

### Common Issues

**"No pseudonym pools provided"**
- **Cause**: Library called without pools
- **Solution**: Use `with_pools()` method or update CLI

**"Empty pools provided"**  
- **Cause**: Pool generation failed or empty arrays provided
- **Solution**: Regenerate pools with `--generate-pools`

**"Inconsistent results across systems"**
- **Cause**: Different pools or seeds used  
- **Solution**: Share pools via `--export-pools` / `--import-pools`

**"Permission denied" on pool files**
- **Cause**: Incorrect file permissions on ~/.anon/  
- **Solution**: `chmod 700 ~/.anon && chmod 600 ~/.anon/pools/*.json`

### Debugging Commands

```bash
# Check current pools
$ anon --show-pools
Pools: ~/.anon/pools/default_pools.json
Generated: 2024-01-15 10:30:00
Seed: 12345, Size: 10000

# Validate pools
$ anon --validate-pools  
✅ Pools are valid and loadable

# Reset to defaults
$ anon --generate-pools --force
⚠️  Overwriting existing pools
✅ Generated fresh default pools
```