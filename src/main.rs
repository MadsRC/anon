use anon_sdk::algorithms::entity_anonymization::EntityAnonymization;
use anon_sdk::cli::{Cli, Commands, DetectorType};
use anon_sdk::detection::hybrid::HybridDetector;
use anon_sdk::detection::ner::GlinerDetector;
use anon_sdk::detection::patterns::PatternDetector;
use anon_sdk::detection::{EntityDetector, EntityType};
use anon_sdk::pool_manager::PoolManager;
use clap::Parser;
use std::fs;
use std::io::{self, Read};

fn main() -> anon_sdk::Result<()> {
    let cli = Cli::parse();
    let pool_manager = PoolManager::new()?;

    match cli.command {
        Some(Commands::GeneratePools { size, seed, force }) => {
            handle_generate_pools(&pool_manager, size, seed, force)
        }

        Some(Commands::ExportPools { output }) => handle_export_pools(&pool_manager, &output),

        Some(Commands::ImportPools { input, set_default }) => {
            handle_import_pools(&pool_manager, &input, set_default)
        }

        Some(Commands::ShowPools { file }) => handle_show_pools(&pool_manager, file.as_deref()),

        Some(Commands::ValidatePools { file }) => {
            handle_validate_pools(&pool_manager, file.as_deref())
        }

        None => {
            // Regular anonymization workflow
            process_anonymization(cli, pool_manager)
        }
    }
}

fn handle_generate_pools(
    pool_manager: &PoolManager,
    size: usize,
    seed: Option<u64>,
    force: bool,
) -> anon_sdk::Result<()> {
    if pool_manager.default_pool_path.exists() && !force {
        eprintln!(
            "❌ Default pools already exist at: {}",
            pool_manager.default_pool_path.display()
        );
        eprintln!("💡 Use --force to overwrite or specify a different output path");
        std::process::exit(1);
    }

    // Use persistent seed management for pool generation too
    let actual_seed = pool_manager.get_or_generate_seed(seed)?;
    pool_manager.generate_default_pools(Some(actual_seed), size)?;

    if seed.is_some() {
        eprintln!(
            "✅ Generated {} pseudonyms with explicit seed {} → {}",
            size,
            actual_seed,
            pool_manager.default_pool_path.display()
        );
    } else {
        eprintln!(
            "✅ Generated {} pseudonyms with persistent seed {} → {}",
            size,
            actual_seed,
            pool_manager.default_pool_path.display()
        );
    }

    Ok(())
}

fn handle_export_pools(
    pool_manager: &PoolManager,
    output: &std::path::Path,
) -> anon_sdk::Result<()> {
    let pools = pool_manager.load_pools(None)?;
    pool_manager.save_pools(&pools, output)?;
    eprintln!("✅ Exported pools → {}", output.display());
    Ok(())
}

fn handle_import_pools(
    pool_manager: &PoolManager,
    input: &std::path::Path,
    set_default: bool,
) -> anon_sdk::Result<()> {
    let pools = pool_manager.load_pools(Some(input))?;

    if set_default {
        pool_manager.save_pools(&pools, &pool_manager.default_pool_path)?;
        eprintln!("✅ Imported {} → default pools", input.display());
    } else {
        eprintln!(
            "✅ Pools validated and loaded from {} (not set as default)",
            input.display()
        );
    }

    Ok(())
}

fn handle_show_pools(
    pool_manager: &PoolManager,
    file: Option<&std::path::Path>,
) -> anon_sdk::Result<()> {
    let metadata = pool_manager.get_pool_info(file)?;
    let pool_path = file.unwrap_or(&pool_manager.default_pool_path);

    println!("📊 Pool Information");
    println!("Path: {}", pool_path.display());
    println!(
        "Generated: {}",
        metadata.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!("Version: {}", metadata.version);
    println!("Generator: v{}", metadata.generator_version);
    println!("Pool Size: {}", metadata.pool_size);

    if let Some(seed) = metadata.pool_seed {
        println!("Pool Seed: {}", seed);
    } else {
        println!("Pool Seed: <random>");
    }

    Ok(())
}

fn handle_validate_pools(
    pool_manager: &PoolManager,
    file: Option<&std::path::Path>,
) -> anon_sdk::Result<()> {
    match pool_manager.load_pools(file) {
        Ok(_) => {
            let pool_path = file.unwrap_or(&pool_manager.default_pool_path);
            eprintln!("✅ Pools are valid: {}", pool_path.display());
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Pool validation failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn process_anonymization(cli: Cli, pool_manager: PoolManager) -> anon_sdk::Result<()> {
    // Read input text
    let input_text = match cli.file {
        Some(filename) => fs::read_to_string(&filename).map_err(|e| {
            anon_sdk::AnonError::InvalidInput(format!("Failed to read file '{}': {}", filename, e))
        })?,
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer).map_err(|e| {
                anon_sdk::AnonError::InvalidInput(format!("Failed to read from stdin: {}", e))
            })?;
            buffer
        }
    };

    // Load pools (auto-generate if needed)
    let pools = if let Some(ref pool_path) = cli.pools {
        pool_manager.load_pools(Some(pool_path))?
    } else {
        pool_manager.ensure_pools_exist()?
    };

    // Get or generate persistent seed
    let seed = pool_manager.get_or_generate_seed(cli.seed)?;

    // Create anonymizer with pools and seed
    let mut anonymizer = EntityAnonymization::new().with_pools(pools);
    anonymizer.set_global_strategy(cli.strategy);
    anonymizer.set_seed(seed);

    if cli.debug {
        if cli.seed.is_some() {
            eprintln!("🔑 Using explicit seed: {}", seed);
        } else if pool_manager.has_persistent_seed() {
            eprintln!("🔑 Using persistent seed: {}", seed);
        } else {
            eprintln!("🔑 Generated new persistent seed: {}", seed);
        }
    }

    // Create detector based on CLI selection (defaults to hybrid)
    let mut detector: Box<dyn EntityDetector> = match cli.detector {
        DetectorType::Hybrid => {
            if cli.debug {
                eprintln!(
                    "Using hybrid detection (NER + patterns) with model: {}",
                    cli.model_path
                );
            }
            let mut hybrid = HybridDetector::with_model_dir(&cli.model_path)?;
            if cli.debug {
                hybrid.enable_debug(true);
            }
            Box::new(hybrid)
        }
        DetectorType::Pattern => {
            if cli.debug {
                eprintln!("Using pattern-only detection");
            }
            Box::new(PatternDetector::new()?)
        }
        DetectorType::Ner => {
            if cli.debug {
                eprintln!("Using NER-only detection with model: {}", cli.model_path);
            }
            let tokenizer_path = format!("{}/tokenizer.json", cli.model_path);
            let model_path = format!("{}/model.onnx", cli.model_path);
            let ner = GlinerDetector::new(
                &tokenizer_path,
                &model_path,
                vec![
                    EntityType::Person,
                    EntityType::Organization,
                    EntityType::Location,
                    EntityType::Email,
                ],
            )?;
            Box::new(ner)
        }
    };

    // Debug: show detected entities if debug flag is set
    if cli.debug {
        let entities = detector.detect(&input_text)?;
        eprintln!("Detected entities:");
        for entity in &entities {
            eprintln!(
                "  - {}: '{}' at {}..{} (confidence: {:.3})",
                entity.entity_type, entity.text, entity.start, entity.end, entity.confidence
            );
        }
        eprintln!();
    }

    // Process the text with selected detector
    let anonymized_text = anonymizer.anonymize_text(&input_text, detector.as_mut())?;

    // Print debug output if enabled (hybrid detector specific)
    if cli.debug {
        if let Some(hybrid_detector) = detector.as_any().downcast_ref::<HybridDetector>() {
            eprintln!(
                "Hybrid Debug output:\n{}",
                hybrid_detector.get_debug_output()
            );
        }
    }

    // Output to stdout
    print!("{}", anonymized_text);

    Ok(())
}
