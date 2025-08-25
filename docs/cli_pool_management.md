# CLI Pool Management Implementation Guide

This document details the implementation of pool management in the CLI layer, including auto-generation, caching, and team sharing workflows.

## Implementation Overview

### Core Components

```rust
// Pool management module
mod pool_manager {
    pub struct PoolManager {
        default_pool_path: PathBuf,
        config_dir: PathBuf,
    }
    
    impl PoolManager {
        pub fn new() -> Result<Self> { ... }
        pub fn ensure_pools_exist(&self) -> Result<PseudonymPools> { ... }
        pub fn load_pools(&self, path: Option<&Path>) -> Result<PseudonymPools> { ... }
        pub fn save_pools(&self, pools: &PseudonymPools, path: &Path) -> Result<()> { ... }
        pub fn generate_default_pools(&self, seed: Option<u64>, size: usize) -> Result<()> { ... }
    }
}
```

### CLI Structure Updates

```rust
#[derive(Parser)]
pub struct Cli {
    // Existing fields...
    
    // Pool management commands
    #[command(subcommand)]
    pub command: Option<Commands>,
    
    // Pool options for anonymization
    #[arg(long, help = "Path to pool file (uses default if not specified)")]
    pub pools: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate new pseudonym pools
    GeneratePools {
        #[arg(long, default_value = "10000")]
        size: usize,
        
        #[arg(long, help = "Seed for deterministic generation")]
        seed: Option<u64>,
        
        #[arg(long, help = "Force overwrite existing pools")]
        force: bool,
    },
    
    /// Export current pools to file
    ExportPools {
        #[arg(help = "Output file path")]
        output: PathBuf,
    },
    
    /// Import pools from file  
    ImportPools {
        #[arg(help = "Input pool file path")]
        input: PathBuf,
        
        #[arg(long, help = "Set as default pools")]
        set_default: bool,
    },
    
    /// Show information about current pools
    ShowPools,
    
    /// Validate pool file integrity
    ValidatePools {
        #[arg(help = "Pool file to validate (uses default if not specified)")]
        file: Option<PathBuf>,
    },
}
```

## Implementation Steps

### Step 1: Directory Structure Setup

```rust
impl PoolManager {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::home_dir()
            .ok_or_else(|| AnonError::InvalidInput("Cannot determine home directory".to_string()))?
            .join(".anon");
            
        let pools_dir = config_dir.join("pools");
        let default_pool_path = pools_dir.join("default_pools.json");
        
        // Create directories if they don't exist
        std::fs::create_dir_all(&pools_dir)
            .map_err(|e| AnonError::InvalidInput(format!("Cannot create config directory: {}", e)))?;
            
        Ok(Self {
            default_pool_path,
            config_dir,
        })
    }
}
```

### Step 2: Auto-Generation Logic

```rust
impl PoolManager {
    pub fn ensure_pools_exist(&self) -> Result<PseudonymPools> {
        if self.default_pool_path.exists() {
            // Load existing pools
            self.load_pools(None)
        } else {
            // Auto-generate default pools
            eprintln!("⚠️  No pools found. Generating default pools (10,000 pseudonyms)...");
            self.generate_default_pools(None, 10000)?;
            eprintln!("✅ Generated pools cached to {}", self.default_pool_path.display());
            self.load_pools(None)
        }
    }
    
    pub fn generate_default_pools(&self, seed: Option<u64>, size: usize) -> Result<()> {
        let seed = seed.unwrap_or_else(|| {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        });
        
        let pools = PseudonymPools::generate_with_seed(seed, size);
        self.save_pools(&pools, &self.default_pool_path)?;
        Ok(())
    }
}
```

### Step 3: Pool Serialization with Metadata

```rust
#[derive(Serialize, Deserialize)]
struct PoolFile {
    metadata: PoolMetadata,
    pools: PseudonymPools,
}

#[derive(Serialize, Deserialize)]
struct PoolMetadata {
    version: String,
    generated_at: String,
    pool_seed: Option<u64>,
    pool_size: usize,
    generator_version: String,
}

impl PoolManager {
    pub fn save_pools(&self, pools: &PseudonymPools, path: &Path) -> Result<()> {
        let metadata = PoolMetadata {
            version: "1.0".to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            pool_seed: None, // Could be tracked if needed
            pool_size: pools.first_names.as_ref().map(|v| v.len()).unwrap_or(0),
            generator_version: env!("CARGO_PKG_VERSION").to_string(),
        };
        
        let pool_file = PoolFile {
            metadata,
            pools: pools.clone(),
        };
        
        let json = serde_json::to_string_pretty(&pool_file)
            .map_err(|e| AnonError::InvalidInput(format!("Serialization error: {}", e)))?;
            
        std::fs::write(path, json)
            .map_err(|e| AnonError::InvalidInput(format!("Cannot write pool file: {}", e)))?;
            
        // Set restrictive permissions (600)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(path)
                .map_err(|e| AnonError::InvalidInput(format!("Cannot read file metadata: {}", e)))?
                .permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(path, perms)
                .map_err(|e| AnonError::InvalidInput(format!("Cannot set file permissions: {}", e)))?;
        }
        
        Ok(())
    }
}
```

### Step 4: Command Processing

```rust
fn main() -> anon_sdk::Result<()> {
    let cli = Cli::parse();
    let pool_manager = PoolManager::new()?;
    
    match cli.command {
        Some(Commands::GeneratePools { size, seed, force }) => {
            if pool_manager.default_pool_path.exists() && !force {
                eprintln!("❌ Default pools already exist. Use --force to overwrite.");
                std::process::exit(1);
            }
            
            pool_manager.generate_default_pools(seed, size)?;
            if let Some(seed) = seed {
                eprintln!("✅ Generated {} pseudonyms with seed {} → {}", 
                    size, seed, pool_manager.default_pool_path.display());
            } else {
                eprintln!("✅ Generated {} pseudonyms → {}", 
                    size, pool_manager.default_pool_path.display());
            }
        },
        
        Some(Commands::ExportPools { output }) => {
            let pools = pool_manager.load_pools(None)?;
            pool_manager.save_pools(&pools, &output)?;
            eprintln!("✅ Exported pools → {}", output.display());
        },
        
        Some(Commands::ImportPools { input, set_default }) => {
            let pools = pool_manager.load_pools(Some(&input))?;
            
            if set_default {
                pool_manager.save_pools(&pools, &pool_manager.default_pool_path)?;
                eprintln!("✅ Imported {} → default pools", input.display());
            } else {
                eprintln!("✅ Pools loaded from {} (not set as default)", input.display());
            }
        },
        
        Some(Commands::ShowPools) => {
            show_pool_info(&pool_manager)?;
        },
        
        Some(Commands::ValidatePools { file }) => {
            validate_pools(&pool_manager, file.as_deref())?;
        },
        
        None => {
            // Regular anonymization workflow
            process_anonymization(cli, pool_manager)?;
        }
    }
    
    Ok(())
}
```

### Step 5: Anonymization Integration

```rust
fn process_anonymization(cli: Cli, pool_manager: PoolManager) -> anon_sdk::Result<()> {
    // Read input
    let input_text = read_input(&cli.file)?;
    
    // Load pools (auto-generate if needed)
    let pools = if let Some(pool_path) = &cli.pools {
        pool_manager.load_pools(Some(pool_path))?
    } else {
        pool_manager.ensure_pools_exist()?
    };
    
    // Create anonymizer with pools
    let mut anonymizer = EntityAnonymization::new().with_pools(pools);
    anonymizer.set_global_strategy(cli.strategy);
    
    if let Some(seed) = cli.seed {
        anonymizer.set_seed(seed);
    }
    
    // Create detector
    let detector: Box<dyn EntityDetector> = create_detector(cli.detector)?;
    
    // Process and output
    let anonymized_text = anonymizer.anonymize_text(&input_text, detector.as_ref())?;
    print!("{}", anonymized_text);
    
    Ok(())
}
```

## Error Handling Strategy

### Graceful Degradation

```rust
impl PoolManager {
    pub fn ensure_pools_exist(&self) -> Result<PseudonymPools> {
        match self.load_default_pools() {
            Ok(pools) => Ok(pools),
            Err(_) => {
                eprintln!("⚠️  Cannot load existing pools, regenerating...");
                self.generate_default_pools(None, 10000)?;
                self.load_default_pools()
            }
        }
    }
}
```

### User-Friendly Messages

```rust
fn handle_pool_error(error: &AnonError) -> ! {
    match error {
        AnonError::InvalidInput(msg) if msg.contains("No pseudonym pools") => {
            eprintln!("❌ Pool Error: No pools available");
            eprintln!("💡 Try: anon --generate-pools");
            std::process::exit(1);
        },
        AnonError::InvalidInput(msg) if msg.contains("Empty") => {
            eprintln!("❌ Pool Error: Pools are empty or corrupted");  
            eprintln!("💡 Try: anon --generate-pools --force");
            std::process::exit(1);
        },
        _ => {
            eprintln!("❌ Unexpected error: {}", error);
            std::process::exit(1);
        }
    }
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_pool_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".anon");
        
        // Test directory creation
        let manager = PoolManager::with_config_dir(config_path).unwrap();
        assert!(manager.config_dir.exists());
    }
    
    #[test] 
    fn test_auto_generation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PoolManager::with_config_dir(temp_dir.path().join(".anon")).unwrap();
        
        // Should auto-generate on first call
        let pools = manager.ensure_pools_exist().unwrap();
        assert!(pools.first_names.is_some());
        assert!(manager.default_pool_path.exists());
    }
    
    #[test]
    fn test_deterministic_generation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PoolManager::with_config_dir(temp_dir.path().join(".anon")).unwrap();
        
        manager.generate_default_pools(Some(12345), 100).unwrap();
        let pools1 = manager.load_pools(None).unwrap();
        
        manager.generate_default_pools(Some(12345), 100).unwrap();  
        let pools2 = manager.load_pools(None).unwrap();
        
        assert_eq!(pools1, pools2);
    }
}
```

### Integration Tests

```rust
#[test]
fn test_cli_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let home_dir = temp_dir.path();
    
    // Set temporary home directory
    std::env::set_var("HOME", home_dir);
    
    // Test auto-generation on first run
    let output = Command::new("target/debug/anon")
        .arg("--help") // Non-anonymization command to trigger pool check
        .output()
        .unwrap();
        
    assert!(home_dir.join(".anon/pools/default_pools.json").exists());
}
```

## Dependencies Required

Add to Cargo.toml:

```toml
[dependencies]
# Existing dependencies...
dirs = "5.0"           # Home directory detection
chrono = "0.4"         # Timestamp generation  
clap = { version = "4.4", features = ["derive"] }

[dev-dependencies]
tempfile = "3.8"       # Temporary directories for testing
```

## Migration Path

### Backward Compatibility

The CLI will maintain backward compatibility:

1. **Existing workflows** continue to work (auto-generation on first run)
2. **New features** are opt-in (`--pools`, `--generate-pools`, etc.)
3. **Error messages** guide users to new functionality

### Rollout Strategy

1. **Phase 1**: Implement auto-generation (seamless for existing users)
2. **Phase 2**: Add pool management commands (power users)
3. **Phase 3**: Add team sharing features (enterprise users)

This implementation provides a robust, user-friendly pool management system that enhances security while maintaining ease of use.