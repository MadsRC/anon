use crate::algorithms::entity_anonymization::AnonymizationStrategy;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

fn default_model_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
    format!("{}/.anon/models", home)
}

#[derive(Debug, Clone, PartialEq)]
pub enum DetectorType {
    Hybrid,
    Pattern,
    Ner,
}

impl std::str::FromStr for DetectorType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "hybrid" => Ok(DetectorType::Hybrid),
            "pattern" => Ok(DetectorType::Pattern),
            "ner" => Ok(DetectorType::Ner),
            _ => Err(format!(
                "Invalid detector: {}. Valid options: hybrid, pattern, ner",
                s
            )),
        }
    }
}

impl std::fmt::Display for DetectorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DetectorType::Hybrid => write!(f, "hybrid"),
            DetectorType::Pattern => write!(f, "pattern"),
            DetectorType::Ner => write!(f, "ner"),
        }
    }
}

#[derive(Parser)]
#[command(name = "anon")]
#[command(about = "A CLI tool for anonymizing sensitive data in text")]
#[command(version)]
pub struct Cli {
    /// Pool management commands
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Input file to process. If not provided, reads from stdin
    #[arg(short, long)]
    pub file: Option<String>,

    /// Anonymization strategy to use for all detected entities
    #[arg(short, long, default_value = "pseudonymize")]
    pub strategy: AnonymizationStrategy,

    /// Detection method to use for finding entities
    #[arg(short, long, default_value = "hybrid")]
    pub detector: DetectorType,

    /// Seed for deterministic pseudonymization (enables consistent and reversible pseudonyms)
    #[arg(long)]
    pub seed: Option<u64>,

    /// Path to pool file (uses default if not specified)
    #[arg(long)]
    pub pools: Option<PathBuf>,

    /// Enable debug logging for detection process
    #[arg(long)]
    pub debug: bool,

    /// Path to GLiNER model directory (contains tokenizer.json and model.onnx)
    #[arg(long, default_value_t = default_model_path())]
    pub model_path: String,
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
    ShowPools {
        #[arg(help = "Pool file to show info for (uses default if not specified)")]
        file: Option<PathBuf>,
    },

    /// Validate pool file integrity
    ValidatePools {
        #[arg(help = "Pool file to validate (uses default if not specified)")]
        file: Option<PathBuf>,
    },
}
