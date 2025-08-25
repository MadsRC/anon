use anon_sdk::cli::Cli;
use anon_sdk::detection::EntityType;
use anon_sdk::detection::ner::GlinerDetector;
use clap::Parser;

#[test]
fn test_ner_always_available_without_feature_flags() {
    // This test verifies that NER functionality works without requiring --features ner

    // Should be able to create GlinerDetector without feature compilation
    let detector_result = GlinerDetector::new(
        "models/gliner/gliner_small-v2.1/tokenizer.json",
        "models/gliner/gliner_small-v2.1/model.onnx",
        vec![EntityType::Person, EntityType::Organization],
    );

    // The test should NOT get a compile-time "NER feature not enabled" error
    // It may get a runtime error for missing model files, but that's different
    match detector_result {
        Ok(_) => {
            // Great! NER detector created successfully
            assert!(true);
        }
        Err(e) => {
            // Should be a runtime error (missing files), not a compile-time feature error
            let error_msg = format!("{}", e);
            assert!(
                !error_msg.contains("NER feature not enabled"),
                "Got compile-time feature error: {}",
                error_msg
            );
        }
    }
}

#[test]
fn test_model_path_default_should_be_home_anon_models() {
    // RED phase: This test should fail initially because the current default is "models"
    // We want the default to be "$HOME/.anon/models"

    let args = vec!["anon"];
    let cli = Cli::parse_from(args);

    let expected_path = format!(
        "{}/.anon/models",
        std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string())
    );
    assert_eq!(cli.model_path, expected_path);
}
