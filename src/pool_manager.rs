use crate::algorithms::entity_anonymization::PseudonymPools;
use crate::{AnonError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use rand::random;

#[derive(Serialize, Deserialize)]
pub struct PoolFile {
    pub metadata: PoolMetadata,
    pub pools: PseudonymPools,
}

#[derive(Serialize, Deserialize)]
pub struct PoolMetadata {
    pub version: String,
    pub generated_at: DateTime<Utc>,
    pub pool_seed: Option<u64>,
    pub pool_size: usize,
    pub generator_version: String,
}

pub struct PoolManager {
    pub default_pool_path: PathBuf,
    pub config_dir: PathBuf,
}

impl PoolManager {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::home_dir()
            .ok_or_else(|| AnonError::InvalidInput("Cannot determine home directory".to_string()))?
            .join(".anon");

        let pools_dir = config_dir.join("pools");
        let default_pool_path = pools_dir.join("default_pools.json");

        // Create directories if they don't exist
        std::fs::create_dir_all(&pools_dir).map_err(|e| {
            AnonError::InvalidInput(format!("Cannot create config directory: {}", e))
        })?;

        Ok(Self {
            default_pool_path,
            config_dir,
        })
    }

    #[cfg(test)]
    pub fn with_config_dir(config_dir: PathBuf) -> Result<Self> {
        let pools_dir = config_dir.join("pools");
        let default_pool_path = pools_dir.join("default_pools.json");

        std::fs::create_dir_all(&pools_dir).map_err(|e| {
            AnonError::InvalidInput(format!("Cannot create config directory: {}", e))
        })?;

        Ok(Self {
            default_pool_path,
            config_dir,
        })
    }

    pub fn ensure_pools_exist(&self) -> Result<PseudonymPools> {
        if self.default_pool_path.exists() {
            // Load existing pools
            self.load_pools(None)
        } else {
            // Auto-generate default pools
            eprintln!("⚠️  No pools found. Generating default pools (10,000 pseudonyms)...");
            self.generate_default_pools(None, 10000)?;
            eprintln!(
                "✅ Generated pools cached to {}",
                self.default_pool_path.display()
            );
            self.load_pools(None)
        }
    }

    pub fn load_pools(&self, path: Option<&Path>) -> Result<PseudonymPools> {
        let pool_path = path.unwrap_or(&self.default_pool_path);

        let contents = std::fs::read_to_string(pool_path).map_err(|e| {
            AnonError::InvalidInput(format!(
                "Cannot read pool file '{}': {}",
                pool_path.display(),
                e
            ))
        })?;

        let pool_file: PoolFile = serde_json::from_str(&contents).map_err(|e| {
            AnonError::InvalidInput(format!(
                "Cannot parse pool file '{}': {}",
                pool_path.display(),
                e
            ))
        })?;

        // Validate pools are not empty
        self.validate_pools(&pool_file.pools)?;

        Ok(pool_file.pools)
    }

    pub fn save_pools(&self, pools: &PseudonymPools, path: &Path) -> Result<()> {
        let metadata = PoolMetadata {
            version: "1.0".to_string(),
            generated_at: Utc::now(),
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

        // Set restrictive permissions (600) on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(path) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o600);
                let _ = std::fs::set_permissions(path, perms);
            }
        }

        Ok(())
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

    pub fn get_pool_info(&self, path: Option<&Path>) -> Result<PoolMetadata> {
        let pool_path = path.unwrap_or(&self.default_pool_path);

        let contents = std::fs::read_to_string(pool_path).map_err(|e| {
            AnonError::InvalidInput(format!(
                "Cannot read pool file '{}': {}",
                pool_path.display(),
                e
            ))
        })?;

        let pool_file: PoolFile = serde_json::from_str(&contents).map_err(|e| {
            AnonError::InvalidInput(format!(
                "Cannot parse pool file '{}': {}",
                pool_path.display(),
                e
            ))
        })?;

        Ok(pool_file.metadata)
    }

    fn validate_pools(&self, pools: &PseudonymPools) -> Result<()> {
        // Check that required pools exist and are not empty
        if let Some(ref names) = pools.first_names {
            if names.is_empty() {
                return Err(AnonError::InvalidInput(
                    "First names pool is empty".to_string(),
                ));
            }
        }

        if let Some(ref names) = pools.last_names {
            if names.is_empty() {
                return Err(AnonError::InvalidInput(
                    "Last names pool is empty".to_string(),
                ));
            }
        }

        if let Some(ref orgs) = pools.organizations {
            if orgs.is_empty() {
                return Err(AnonError::InvalidInput(
                    "Organizations pool is empty".to_string(),
                ));
            }
        }

        if let Some(ref domains) = pools.email_domains {
            if domains.is_empty() {
                return Err(AnonError::InvalidInput(
                    "Email domains pool is empty".to_string(),
                ));
            }
        }

        if let Some(ref locations) = pools.locations {
            if locations.is_empty() {
                return Err(AnonError::InvalidInput(
                    "Locations pool is empty".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Get or generate a persistent seed for anonymization
    /// 
    /// This function:
    /// 1. Returns explicit seed if provided
    /// 2. Returns existing persisted seed if available
    /// 3. Generates, saves, and returns new random seed otherwise
    pub fn get_or_generate_seed(&self, explicit_seed: Option<u64>) -> Result<u64> {
        // Use explicit seed if provided
        if let Some(seed) = explicit_seed {
            return Ok(seed);
        }

        let seed_file = self.config_dir.join("seed");

        // Try to load existing seed
        if let Ok(existing_seed) = std::fs::read_to_string(&seed_file) {
            if let Ok(seed) = existing_seed.trim().parse::<u64>() {
                return Ok(seed);
            }
        }

        // Generate new cryptographically secure random seed
        let new_seed = random::<u64>();
        
        // Save the seed for future use
        std::fs::write(&seed_file, new_seed.to_string()).map_err(|e| {
            AnonError::InvalidInput(format!(
                "Failed to save seed to {}: {}",
                seed_file.display(),
                e
            ))
        })?;

        Ok(new_seed)
    }

    /// Get the path to the seed file
    pub fn seed_file_path(&self) -> PathBuf {
        self.config_dir.join("seed")
    }

    /// Check if a persistent seed exists
    pub fn has_persistent_seed(&self) -> bool {
        self.seed_file_path().exists()
    }

    /// Remove the persistent seed (useful for testing or reset)
    pub fn remove_persistent_seed(&self) -> Result<()> {
        let seed_file = self.seed_file_path();
        if seed_file.exists() {
            std::fs::remove_file(&seed_file).map_err(|e| {
                AnonError::InvalidInput(format!(
                    "Failed to remove seed file {}: {}",
                    seed_file.display(),
                    e
                ))
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_pool_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".anon");

        let _manager = PoolManager::with_config_dir(config_path.clone()).unwrap();
        assert!(config_path.exists());
        assert!(config_path.join("pools").exists());
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

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PoolManager::with_config_dir(temp_dir.path().join(".anon")).unwrap();

        let pools = PseudonymPools::generate_with_seed(42, 50);
        let custom_path = temp_dir.path().join("custom_pools.json");

        manager.save_pools(&pools, &custom_path).unwrap();
        let loaded_pools = manager.load_pools(Some(&custom_path)).unwrap();

        assert_eq!(pools, loaded_pools);
    }

    #[test]
    fn test_pool_validation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PoolManager::with_config_dir(temp_dir.path().join(".anon")).unwrap();

        // Empty pools should fail validation
        let empty_pools = PseudonymPools::new()
            .with_first_names(vec![])
            .with_last_names(vec!["Test".to_string()]);

        let result = manager.validate_pools(&empty_pools);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_seed_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PoolManager::with_config_dir(temp_dir.path().join(".anon")).unwrap();

        // Should not have a persistent seed initially
        assert!(!manager.has_persistent_seed());

        // First call should generate and persist a seed
        let seed1 = manager.get_or_generate_seed(None).unwrap();
        assert!(manager.has_persistent_seed());

        // Second call should return the same seed
        let seed2 = manager.get_or_generate_seed(None).unwrap();
        assert_eq!(seed1, seed2);

        // Explicit seed should take priority
        let explicit_seed = 99999u64;
        let seed3 = manager.get_or_generate_seed(Some(explicit_seed)).unwrap();
        assert_eq!(seed3, explicit_seed);

        // But persistent seed should still be there for future calls
        let seed4 = manager.get_or_generate_seed(None).unwrap();
        assert_eq!(seed4, seed1); // Should return original persistent seed
    }

    #[test]
    fn test_seed_removal() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PoolManager::with_config_dir(temp_dir.path().join(".anon")).unwrap();

        // Generate a persistent seed
        let seed1 = manager.get_or_generate_seed(None).unwrap();
        assert!(manager.has_persistent_seed());

        // Remove the seed
        manager.remove_persistent_seed().unwrap();
        assert!(!manager.has_persistent_seed());

        // Should generate a new seed
        let seed2 = manager.get_or_generate_seed(None).unwrap();
        assert_ne!(seed1, seed2); // Should be different (very high probability)
        assert!(manager.has_persistent_seed()); // Should be persisted again
    }

    #[test]
    fn test_seed_file_corruption_recovery() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PoolManager::with_config_dir(temp_dir.path().join(".anon")).unwrap();

        // Write invalid content to seed file
        let seed_file = manager.seed_file_path();
        std::fs::write(&seed_file, "invalid_content").unwrap();

        // Should generate new seed when file is corrupted
        let seed = manager.get_or_generate_seed(None).unwrap();
        assert!(seed > 0); // Should be a valid u64

        // Should overwrite with valid seed
        let content = std::fs::read_to_string(&seed_file).unwrap();
        assert_eq!(content.trim().parse::<u64>().unwrap(), seed);
    }
}
