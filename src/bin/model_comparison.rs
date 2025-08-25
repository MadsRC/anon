use anon_sdk::evaluation::confidence_analysis::ConfidenceAnalyzer;
use anon_sdk::evaluation::model_evaluation::{
    ModelConfig, ModelEvaluator, create_standard_test_cases,
};
use anon_sdk::evaluation::performance_benchmarks::PerformanceTester;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 COMPREHENSIVE GLINER MODEL COMPARISON");
    println!("========================================");

    // Check if models exist
    let models_exist = check_models_available();
    if !models_exist {
        println!("❌ Models not found. Run 'make download-models' first.");
        return Ok(());
    }

    println!("📊 Running accuracy comparison tests...");
    run_accuracy_comparison();

    println!("\n⚡ Running performance comparison tests...");
    run_performance_comparison();

    println!("\n🎯 Running confidence threshold analysis...");
    run_confidence_threshold_analysis();

    println!("\n📈 Running quantized model analysis...");
    run_quantized_model_analysis();

    println!(
        "\n✅ Model comparison complete! Check results above and docs/gliner_model_comparison.md for detailed recommendations."
    );

    Ok(())
}

fn check_models_available() -> bool {
    use std::path::Path;

    let required_files = vec![
        "models/gliner/gliner_small-v2.1/model.onnx",
        "models/gliner/gliner_small-v2.1/model_quantized.onnx",
        "models/gliner/gliner_small-v2.1/tokenizer.json",
        "models/gliner/gliner-x-small/model.onnx",
        "models/gliner/gliner-x-small/model_quantized.onnx",
        "models/gliner/gliner-x-small/tokenizer.json",
    ];

    for file in &required_files {
        if !Path::new(file).exists() {
            return false;
        }
    }

    true
}

fn run_accuracy_comparison() {
    let mut evaluator = ModelEvaluator::new();

    // Add all model variants (regular and quantized)
    evaluator.add_model(ModelConfig::gliner_small());
    evaluator.add_model(ModelConfig {
        name: "GLiNER Small v2.1 (Quantized)".to_string(),
        tokenizer_path: "models/gliner/gliner_small-v2.1/tokenizer.json".to_string(),
        model_path: "models/gliner/gliner_small-v2.1/model_quantized.onnx".to_string(),
    });
    evaluator.add_model(ModelConfig {
        name: "GLiNER X-Small".to_string(),
        tokenizer_path: "models/gliner/gliner-x-small/tokenizer.json".to_string(),
        model_path: "models/gliner/gliner-x-small/model.onnx".to_string(),
    });
    evaluator.add_model(ModelConfig {
        name: "GLiNER X-Small (Quantized)".to_string(),
        tokenizer_path: "models/gliner/gliner-x-small/tokenizer.json".to_string(),
        model_path: "models/gliner/gliner-x-small/model_quantized.onnx".to_string(),
    });

    // Add test cases
    for test_case in create_standard_test_cases() {
        evaluator.add_test_case(test_case);
    }

    let results = evaluator.evaluate_all_models(0.5);

    if !results.is_empty() {
        evaluator.print_summary_report(&results);

        // Calculate model comparison metrics
        let mut model_scores: HashMap<String, Vec<f32>> = HashMap::new();
        for result in &results {
            model_scores
                .entry(result.model_name.clone())
                .or_insert_with(Vec::new)
                .push(result.f1_score);
        }

        println!("\n🏆 MODEL COMPARISON SUMMARY:");
        println!("=============================");

        for (model_name, f1_scores) in &model_scores {
            let avg_f1 = f1_scores.iter().sum::<f32>() / f1_scores.len() as f32;
            let min_f1 = f1_scores.iter().fold(f32::INFINITY, |a, &b| a.min(b));
            let max_f1 = f1_scores.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

            println!("📊 {}:", model_name);
            println!("   Average F1: {:.1}%", avg_f1 * 100.0);
            println!("   Range: {:.1}% - {:.1}%", min_f1 * 100.0, max_f1 * 100.0);
            println!("   Test cases: {}", f1_scores.len());
        }

        // Find best model
        let best_model = model_scores
            .iter()
            .max_by(|a, b| {
                let avg_a = a.1.iter().sum::<f32>() / a.1.len() as f32;
                let avg_b = b.1.iter().sum::<f32>() / b.1.len() as f32;
                avg_a.partial_cmp(&avg_b).unwrap()
            })
            .map(|(name, scores)| (name, scores.iter().sum::<f32>() / scores.len() as f32));

        if let Some((best_name, best_score)) = best_model {
            println!(
                "\n🥇 Winner: {} ({:.1}% avg F1)",
                best_name,
                best_score * 100.0
            );
        }
    } else {
        println!("⚠️  No models available for comparison. Check model files and paths.");
    }
}

fn run_performance_comparison() {
    let mut tester = PerformanceTester::new();

    // Add all model variants (regular and quantized)
    tester.add_model(
        "GLiNER Small v2.1".to_string(),
        "models/gliner/gliner_small-v2.1/tokenizer.json".to_string(),
        "models/gliner/gliner_small-v2.1/model.onnx".to_string(),
    );

    tester.add_model(
        "GLiNER Small v2.1 (Quantized)".to_string(),
        "models/gliner/gliner_small-v2.1/tokenizer.json".to_string(),
        "models/gliner/gliner_small-v2.1/model_quantized.onnx".to_string(),
    );

    tester.add_model(
        "GLiNER X-Small".to_string(),
        "models/gliner/gliner-x-small/tokenizer.json".to_string(),
        "models/gliner/gliner-x-small/model.onnx".to_string(),
    );

    tester.add_model(
        "GLiNER X-Small (Quantized)".to_string(),
        "models/gliner/gliner-x-small/tokenizer.json".to_string(),
        "models/gliner/gliner-x-small/model_quantized.onnx".to_string(),
    );

    // Use a smaller test set for quicker comparison
    tester.with_synthetic_texts(10).with_real_world_texts();

    let results = tester.run_comprehensive_benchmarks();

    if !results.is_empty() {
        tester.print_performance_report(&results);

        // Find throughput results for comparison
        let throughput_results: Vec<_> = results
            .iter()
            .filter(|r| r.test_name == "Throughput Test")
            .collect();

        if throughput_results.len() >= 2 {
            println!("\n🏁 PERFORMANCE WINNER:");
            println!("=====================");

            let fastest = throughput_results
                .iter()
                .max_by(|a, b| a.chars_per_second.partial_cmp(&b.chars_per_second).unwrap());

            let lowest_latency = throughput_results
                .iter()
                .min_by(|a, b| a.avg_latency_per_text.cmp(&b.avg_latency_per_text));

            if let Some(fastest) = fastest {
                println!(
                    "🚀 Highest Throughput: {} ({:.0} chars/sec)",
                    fastest.model_name, fastest.chars_per_second
                );
            }

            if let Some(lowest) = lowest_latency {
                println!(
                    "⚡ Lowest Latency: {} ({:.0}ms avg)",
                    lowest.model_name,
                    lowest.avg_latency_per_text.as_millis()
                );
            }
        }
    } else {
        println!("⚠️  No models available for performance comparison.");
    }
}

fn run_confidence_threshold_analysis() {
    let mut analyzer = ConfidenceAnalyzer::new();

    // Add all model variants (regular and quantized)
    analyzer.add_model(
        "GLiNER Small v2.1".to_string(),
        "models/gliner/gliner_small-v2.1/tokenizer.json".to_string(),
        "models/gliner/gliner_small-v2.1/model.onnx".to_string(),
    );

    analyzer.add_model(
        "GLiNER Small v2.1 (Quantized)".to_string(),
        "models/gliner/gliner_small-v2.1/tokenizer.json".to_string(),
        "models/gliner/gliner_small-v2.1/model_quantized.onnx".to_string(),
    );

    analyzer.add_model(
        "GLiNER X-Small".to_string(),
        "models/gliner/gliner-x-small/tokenizer.json".to_string(),
        "models/gliner/gliner-x-small/model.onnx".to_string(),
    );

    analyzer.add_model(
        "GLiNER X-Small (Quantized)".to_string(),
        "models/gliner/gliner-x-small/tokenizer.json".to_string(),
        "models/gliner/gliner-x-small/model_quantized.onnx".to_string(),
    );

    // Use fewer thresholds for quicker testing
    analyzer = analyzer.with_thresholds(vec![0.3, 0.5, 0.7]);
    analyzer.with_ground_truth_data();

    let results = analyzer.analyze_all_models();

    if !results.is_empty() {
        analyzer.print_confidence_analysis(&results);

        // Find optimal thresholds per model
        let mut model_thresholds: HashMap<String, (f32, f32)> = HashMap::new(); // (threshold, f1_score)

        for result in &results {
            let current_best = model_thresholds
                .get(&result.model_name)
                .map(|(_, f1)| *f1)
                .unwrap_or(0.0);

            if result.f1_score > current_best {
                model_thresholds.insert(
                    result.model_name.clone(),
                    (result.threshold, result.f1_score),
                );
            }
        }

        println!("\n🎯 OPTIMAL THRESHOLDS COMPARISON:");
        println!("================================");

        for (model_name, (threshold, f1_score)) in model_thresholds {
            println!(
                "📊 {}: threshold {:.1} → F1 {:.1}%",
                model_name,
                threshold,
                f1_score * 100.0
            );
        }
    } else {
        println!("⚠️  No models available for confidence analysis.");
    }
}

fn run_quantized_model_analysis() {
    let mut evaluator = ModelEvaluator::new();

    // Add all quantized models
    evaluator.add_model(ModelConfig {
        name: "GLiNER Small v2.1 (Quantized)".to_string(),
        tokenizer_path: "models/gliner/gliner_small-v2.1/tokenizer.json".to_string(),
        model_path: "models/gliner/gliner_small-v2.1/model_quantized.onnx".to_string(),
    });

    evaluator.add_model(ModelConfig {
        name: "GLiNER X-Small (Quantized)".to_string(),
        tokenizer_path: "models/gliner/gliner-x-small/tokenizer.json".to_string(),
        model_path: "models/gliner/gliner-x-small/model_quantized.onnx".to_string(),
    });

    // Add test cases
    for test_case in create_standard_test_cases() {
        evaluator.add_test_case(test_case);
    }

    let results = evaluator.evaluate_all_models(0.5);

    if !results.is_empty() {
        evaluator.print_summary_report(&results);

        println!("\n🏆 QUANTIZED MODEL RANKINGS:");
        println!("===========================");

        let mut model_scores: HashMap<String, f32> = HashMap::new();
        for result in &results {
            let entry = model_scores.entry(result.model_name.clone()).or_insert(0.0);
            *entry += result.f1_score;
        }

        // Calculate average scores and sort
        let test_case_count = create_standard_test_cases().len() as f32;
        let mut ranked_models: Vec<_> = model_scores
            .iter()
            .map(|(name, total_score)| (name.clone(), total_score / test_case_count))
            .collect();
        ranked_models.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        for (i, (model_name, avg_f1)) in ranked_models.iter().enumerate() {
            println!("{}. {} - {:.1}% F1", i + 1, model_name, avg_f1 * 100.0);
        }
    } else {
        println!("⚠️  No quantized models available for testing.");
    }
}
