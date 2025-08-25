use anon_sdk::algorithms::entity_anonymization::{EntityAnonymization, ReplacementStrategy};
use anon_sdk::detection::{EntityDetector, EntityType, ner::GlinerDetector};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 GLiNER NER Integration Demo\n");

    // Note: You would need to download GLiNER models from Hugging Face
    // Example models:
    // - gliner-small-2.1 (span mode)
    // - gliner-multitask-large-0.5 (token mode)

    let tokenizer_path = "models/gliner/gliner_small-v2.1/tokenizer.json";
    let model_path = "models/gliner/gliner_small-v2.1/model.onnx";

    let entity_types = vec![
        EntityType::Person,
        EntityType::Location,
        EntityType::Organization,
        EntityType::Custom("product".to_string()),
        EntityType::Custom("event".to_string()),
    ];

    println!("🔧 Loading GLiNER model...");
    let mut detector = GlinerDetector::new(tokenizer_path, model_path, entity_types)?
        .with_confidence_threshold(0.7)?;

    let sample_text = r#"
    John Smith from Microsoft will be attending the Apple Keynote event in San Francisco next week.
    He can be reached at john.smith@microsoft.com or by calling his office at (555) 123-4567.
    The event will showcase the new iPhone 15 and will be held at the Moscone Center.
    Other attendees include representatives from Google, Amazon, and Tesla.
    "#;

    println!("📄 Original Text:\n{}", sample_text);

    println!("🔍 Detecting entities with GLiNER...");
    let entities = detector.detect(sample_text)?;

    println!("\n🎯 Detected Entities:");
    for (i, entity) in entities.iter().enumerate() {
        println!(
            "  {}. {} (confidence: {:.2}): '{}'",
            i + 1,
            entity.entity_type,
            entity.confidence,
            entity.text
        );
    }

    println!("\n🎭 Applying anonymization...");
    let mut anonymizer = EntityAnonymization::new();

    // Configure different strategies for different entity types
    anonymizer.add_replacement_strategy(EntityType::Person, ReplacementStrategy::Pseudonymize);
    anonymizer.add_replacement_strategy(
        EntityType::Location,
        ReplacementStrategy::Generalize("LOCATION".to_string()),
    );
    anonymizer.add_replacement_strategy(
        EntityType::Organization,
        ReplacementStrategy::Generalize("ORG".to_string()),
    );
    anonymizer.add_replacement_strategy(
        EntityType::Custom("product".to_string()),
        ReplacementStrategy::Generalize("PRODUCT".to_string()),
    );
    anonymizer.add_replacement_strategy(
        EntityType::Custom("event".to_string()),
        ReplacementStrategy::Generalize("EVENT".to_string()),
    );

    let anonymized_text = anonymizer.anonymize_text(sample_text, &mut detector)?;

    println!("✅ Anonymized Text:\n{}", anonymized_text);

    println!("\n🚀 GPU Acceleration Example:");
    println!("To use GPU acceleration:");
    println!("let gpu_detector = GlinerDetector::with_gpu_acceleration(");
    println!("    tokenizer_path, model_path, entity_types)?;");

    Ok(())
}
