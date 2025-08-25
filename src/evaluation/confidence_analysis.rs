use crate::detection::{EntityDetector, EntityType, ner::GlinerDetector};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ConfidenceAnalysisResult {
    pub model_name: String,
    pub threshold: f32,
    #[allow(dead_code)]
    pub test_case: String,
    pub detected_entities: Vec<EntityResult>,
    pub precision: f32,
    pub recall: f32,
    pub f1_score: f32,
    pub total_detections: usize,
    pub correct_detections: usize,
    pub false_positives: usize,
    pub false_negatives: usize,
}

#[derive(Debug, Clone)]
pub struct EntityResult {
    #[allow(dead_code)]
    pub entity_type: EntityType,
    #[allow(dead_code)]
    pub text: String,
    pub confidence: f32,
    #[allow(dead_code)]
    pub start: usize,
    #[allow(dead_code)]
    pub end: usize,
    #[allow(dead_code)]
    pub is_correct: bool,
    #[allow(dead_code)]
    pub expected_match: Option<String>, // What this should have matched
}

#[derive(Debug, Clone)]
pub struct GroundTruth {
    pub text: String,
    pub expected_entities: Vec<GroundTruthEntity>,
}

#[derive(Debug, Clone)]
pub struct GroundTruthEntity {
    pub entity_type: EntityType,
    pub text: String,
    pub start: usize,
    pub end: usize,
}

pub struct ConfidenceAnalyzer {
    models: Vec<(String, String, String)>, // (name, tokenizer, model)
    ground_truth_data: Vec<GroundTruth>,
    thresholds: Vec<f32>,
}

impl Default for ConfidenceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfidenceAnalyzer {
    pub fn new() -> Self {
        Self {
            models: Vec::new(),
            ground_truth_data: Vec::new(),
            thresholds: vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9],
        }
    }

    pub fn add_model(&mut self, name: String, tokenizer_path: String, model_path: String) {
        self.models.push((name, tokenizer_path, model_path));
    }

    pub fn with_thresholds(mut self, thresholds: Vec<f32>) -> Self {
        self.thresholds = thresholds;
        self
    }

    pub fn with_ground_truth_data(&mut self) -> &mut Self {
        self.ground_truth_data = self.create_ground_truth_dataset();
        self
    }

    fn create_ground_truth_dataset(&self) -> Vec<GroundTruth> {
        vec![
            GroundTruth {
                text: "Contact John Smith at Microsoft via john.smith@microsoft.com or (425) 555-0123.".to_string(),
                expected_entities: vec![
                    GroundTruthEntity { entity_type: EntityType::Person, text: "John Smith".to_string(), start: 8, end: 18 },
                    GroundTruthEntity { entity_type: EntityType::Organization, text: "Microsoft".to_string(), start: 22, end: 31 },
                    GroundTruthEntity { entity_type: EntityType::Email, text: "john.smith@microsoft.com".to_string(), start: 36, end: 60 },
                    GroundTruthEntity { entity_type: EntityType::PhoneNumber, text: "(425) 555-0123".to_string(), start: 64, end: 78 },
                ],
            },
            GroundTruth {
                text: "Dr. Sarah Johnson from Stanford University will speak in San Francisco.".to_string(),
                expected_entities: vec![
                    GroundTruthEntity { entity_type: EntityType::Person, text: "Sarah Johnson".to_string(), start: 4, end: 17 },
                    GroundTruthEntity { entity_type: EntityType::Organization, text: "Stanford University".to_string(), start: 23, end: 42 },
                    GroundTruthEntity { entity_type: EntityType::Location, text: "San Francisco".to_string(), start: 57, end: 70 },
                ],
            },
            GroundTruth {
                text: "Send reports to admin@company.org and backup@test.co.uk immediately.".to_string(),
                expected_entities: vec![
                    GroundTruthEntity { entity_type: EntityType::Email, text: "admin@company.org".to_string(), start: 16, end: 33 },
                    GroundTruthEntity { entity_type: EntityType::Email, text: "backup@test.co.uk".to_string(), start: 38, end: 55 },
                ],
            },
            GroundTruth {
                text: "The meeting between Alice Cooper and Bob Wilson from Google and Amazon respectively is in New York.".to_string(),
                expected_entities: vec![
                    GroundTruthEntity { entity_type: EntityType::Person, text: "Alice Cooper".to_string(), start: 20, end: 32 },
                    GroundTruthEntity { entity_type: EntityType::Person, text: "Bob Wilson".to_string(), start: 37, end: 47 },
                    GroundTruthEntity { entity_type: EntityType::Organization, text: "Google".to_string(), start: 53, end: 59 },
                    GroundTruthEntity { entity_type: EntityType::Organization, text: "Amazon".to_string(), start: 64, end: 70 },
                    GroundTruthEntity { entity_type: EntityType::Location, text: "New York".to_string(), start: 89, end: 97 },
                ],
            },
            GroundTruth {
                text: "Phone support: (555) 123-4567, (650) 555-0198, +1-800-HELP-NOW".to_string(),
                expected_entities: vec![
                    GroundTruthEntity { entity_type: EntityType::PhoneNumber, text: "(555) 123-4567".to_string(), start: 15, end: 29 },
                    GroundTruthEntity { entity_type: EntityType::PhoneNumber, text: "(650) 555-0198".to_string(), start: 31, end: 45 },
                    GroundTruthEntity { entity_type: EntityType::PhoneNumber, text: "+1-800-HELP-NOW".to_string(), start: 47, end: 62 },
                ],
            },
            // Edge cases
            GroundTruth {
                text: "Dr. John Smith-Johnson, M.D. from UCLA Medical Center".to_string(),
                expected_entities: vec![
                    GroundTruthEntity { entity_type: EntityType::Person, text: "John Smith-Johnson".to_string(), start: 4, end: 22 },
                    GroundTruthEntity { entity_type: EntityType::Organization, text: "UCLA Medical Center".to_string(), start: 35, end: 54 },
                ],
            },
            GroundTruth {
                text: "j.doe@company.com".to_string(), // Just an email
                expected_entities: vec![
                    GroundTruthEntity { entity_type: EntityType::Email, text: "j.doe@company.com".to_string(), start: 0, end: 17 },
                ],
            },
            GroundTruth {
                text: "Call 911 for emergencies".to_string(), // Short phone number
                expected_entities: vec![
                    GroundTruthEntity { entity_type: EntityType::PhoneNumber, text: "911".to_string(), start: 5, end: 8 },
                ],
            },
        ]
    }

    pub fn analyze_all_models(&self) -> Vec<ConfidenceAnalysisResult> {
        let mut results = Vec::new();

        for (model_name, tokenizer_path, model_path) in &self.models {
            println!("🔍 Analyzing confidence thresholds for: {}", model_name);

            let detector_result = GlinerDetector::new(
                tokenizer_path,
                model_path,
                vec![
                    EntityType::Person,
                    EntityType::Organization,
                    EntityType::Location,
                    EntityType::Email,
                    EntityType::PhoneNumber,
                ],
            );

            match detector_result {
                Ok(_detector) => {
                    for &threshold in &self.thresholds {
                        println!("  📊 Testing threshold: {}", threshold);

                        let detector_clone = GlinerDetector::new(
                            tokenizer_path,
                            model_path,
                            vec![
                                EntityType::Person,
                                EntityType::Organization,
                                EntityType::Location,
                                EntityType::Email,
                                EntityType::PhoneNumber,
                            ],
                        );
                        if let Ok(detector_clone) = detector_clone {
                            if let Ok(mut thresholded_detector) =
                                detector_clone.with_confidence_threshold(threshold)
                            {
                                for ground_truth in &self.ground_truth_data {
                                    let result = self.analyze_single_case(
                                        model_name,
                                        threshold,
                                        &mut thresholded_detector,
                                        ground_truth,
                                    );
                                    results.push(result);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Failed to load model {}: {}", model_name, e);
                }
            }
        }

        results
    }

    fn analyze_single_case(
        &self,
        model_name: &str,
        threshold: f32,
        detector: &mut GlinerDetector,
        ground_truth: &GroundTruth,
    ) -> ConfidenceAnalysisResult {
        let detected_entities = detector.detect(&ground_truth.text).unwrap_or_default();

        let mut entity_results = Vec::new();
        let mut correct_detections = 0;
        let mut false_positives = 0;

        // Evaluate each detection
        for detected in &detected_entities {
            let (is_correct, expected_match) =
                self.evaluate_detection(detected, &ground_truth.expected_entities);

            if is_correct {
                correct_detections += 1;
            } else {
                false_positives += 1;
            }

            entity_results.push(EntityResult {
                entity_type: detected.entity_type.clone(),
                text: detected.text.clone(),
                confidence: detected.confidence,
                start: detected.start,
                end: detected.end,
                is_correct,
                expected_match,
            });
        }

        // Calculate false negatives (expected entities not detected)
        let mut false_negatives = 0;
        for expected in &ground_truth.expected_entities {
            let was_detected = detected_entities
                .iter()
                .any(|detected| self.entities_match(detected, expected));

            if !was_detected {
                false_negatives += 1;
            }
        }

        // Calculate metrics
        let precision = if !detected_entities.is_empty() {
            correct_detections as f32 / detected_entities.len() as f32
        } else {
            0.0
        };

        let recall = if !ground_truth.expected_entities.is_empty() {
            correct_detections as f32 / ground_truth.expected_entities.len() as f32
        } else {
            0.0
        };

        let f1_score = if precision + recall > 0.0 {
            2.0 * (precision * recall) / (precision + recall)
        } else {
            0.0
        };

        ConfidenceAnalysisResult {
            model_name: model_name.to_string(),
            threshold,
            test_case: ground_truth.text.clone(),
            detected_entities: entity_results,
            precision,
            recall,
            f1_score,
            total_detections: detected_entities.len(),
            correct_detections,
            false_positives,
            false_negatives,
        }
    }

    fn evaluate_detection(
        &self,
        detected: &crate::detection::DetectedEntity,
        expected_entities: &[GroundTruthEntity],
    ) -> (bool, Option<String>) {
        for expected in expected_entities {
            if self.entities_match(detected, expected) {
                return (true, Some(expected.text.clone()));
            }
        }
        (false, None)
    }

    fn entities_match(
        &self,
        detected: &crate::detection::DetectedEntity,
        expected: &GroundTruthEntity,
    ) -> bool {
        // Must match entity type
        if detected.entity_type != expected.entity_type {
            return false;
        }

        // Check for overlap in text positions (allowing some flexibility)
        let detected_range = detected.start..detected.end;
        let expected_range = expected.start..expected.end;

        // Ranges overlap if start of one is before end of other, for both
        let overlaps =
            detected_range.start < expected_range.end && expected_range.start < detected_range.end;

        if overlaps {
            // Also check if the text content is similar
            let detected_text = detected.text.to_lowercase();
            let expected_text = expected.text.to_lowercase();
            let detected_text = detected_text.trim();
            let expected_text = expected_text.trim();

            // Exact match or one contains the other
            detected_text == expected_text
                || detected_text.contains(expected_text)
                || expected_text.contains(detected_text)
        } else {
            false
        }
    }

    pub fn print_confidence_analysis(&self, results: &[ConfidenceAnalysisResult]) {
        println!("\n🎯 CONFIDENCE THRESHOLD ANALYSIS");
        println!("=================================");

        // Group by model
        let mut model_results: HashMap<String, Vec<&ConfidenceAnalysisResult>> = HashMap::new();
        for result in results {
            model_results
                .entry(result.model_name.clone())
                .or_default()
                .push(result);
        }

        for (model_name, model_results) in &model_results {
            println!("\n📊 {} Analysis:", model_name);
            println!("{}", "─".repeat(60));

            // Group by threshold for this model
            let mut threshold_groups: HashMap<String, Vec<&ConfidenceAnalysisResult>> =
                HashMap::new();
            for result in model_results {
                threshold_groups
                    .entry(format!("{:.1}", result.threshold))
                    .or_default()
                    .push(result);
            }

            println!("\n  📈 Performance by Threshold:");
            println!(
                "  Threshold │ Precision │ Recall │ F1 Score │ Detections │ Correct │ FP │ FN"
            );
            println!(
                "  ──────────┼───────────┼────────┼──────────┼────────────┼─────────┼────┼────"
            );

            for threshold in &self.thresholds {
                let threshold_key = format!("{:.1}", threshold);
                if let Some(threshold_results) = threshold_groups.get(&threshold_key) {
                    let avg_precision = threshold_results.iter().map(|r| r.precision).sum::<f32>()
                        / threshold_results.len() as f32;
                    let avg_recall = threshold_results.iter().map(|r| r.recall).sum::<f32>()
                        / threshold_results.len() as f32;
                    let avg_f1 = threshold_results.iter().map(|r| r.f1_score).sum::<f32>()
                        / threshold_results.len() as f32;
                    let total_detections: usize =
                        threshold_results.iter().map(|r| r.total_detections).sum();
                    let total_correct: usize =
                        threshold_results.iter().map(|r| r.correct_detections).sum();
                    let total_fp: usize = threshold_results.iter().map(|r| r.false_positives).sum();
                    let total_fn: usize = threshold_results.iter().map(|r| r.false_negatives).sum();

                    println!(
                        "  {:>8.1} │ {:>8.1}% │ {:>5.1}% │ {:>7.1}% │ {:>10} │ {:>7} │ {:>2} │ {:>2}",
                        threshold,
                        avg_precision * 100.0,
                        avg_recall * 100.0,
                        avg_f1 * 100.0,
                        total_detections,
                        total_correct,
                        total_fp,
                        total_fn
                    );
                }
            }

            // Find optimal threshold
            let mut best_f1 = 0.0;
            let mut best_threshold = 0.0;

            for threshold in &self.thresholds {
                let threshold_key = format!("{:.1}", threshold);
                if let Some(threshold_results) = threshold_groups.get(&threshold_key) {
                    let avg_f1 = threshold_results.iter().map(|r| r.f1_score).sum::<f32>()
                        / threshold_results.len() as f32;
                    if avg_f1 > best_f1 {
                        best_f1 = avg_f1;
                        best_threshold = *threshold;
                    }
                }
            }

            println!(
                "\n  🏆 Optimal Threshold: {:.1} (F1: {:.1}%)",
                best_threshold,
                best_f1 * 100.0
            );
        }

        // Cross-model comparison
        if model_results.len() > 1 {
            self.print_cross_model_comparison(results);
        }
    }

    fn print_cross_model_comparison(&self, results: &[ConfidenceAnalysisResult]) {
        println!("\n🏆 CROSS-MODEL COMPARISON");
        println!("=========================");

        for &threshold in &[0.3, 0.5, 0.7] {
            println!("\n📊 At Threshold {:.1}:", threshold);

            let threshold_results: Vec<_> = results
                .iter()
                .filter(|r| (r.threshold - threshold).abs() < 0.01)
                .collect();

            if threshold_results.is_empty() {
                continue;
            }

            // Group by model for this threshold
            let mut model_groups: HashMap<String, Vec<&ConfidenceAnalysisResult>> = HashMap::new();
            for result in &threshold_results {
                model_groups
                    .entry(result.model_name.clone())
                    .or_default()
                    .push(result);
            }

            println!("  Model               │ Precision │ Recall │ F1 Score │ Avg Confidence");
            println!("  ────────────────────┼───────────┼────────┼──────────┼───────────────");

            for (model_name, model_results) in model_groups {
                let avg_precision = model_results.iter().map(|r| r.precision).sum::<f32>()
                    / model_results.len() as f32;
                let avg_recall = model_results.iter().map(|r| r.recall).sum::<f32>()
                    / model_results.len() as f32;
                let avg_f1 = model_results.iter().map(|r| r.f1_score).sum::<f32>()
                    / model_results.len() as f32;

                // Calculate average confidence of detections
                let all_confidences: Vec<f32> = model_results
                    .iter()
                    .flat_map(|r| r.detected_entities.iter().map(|e| e.confidence))
                    .collect();
                let avg_confidence = if !all_confidences.is_empty() {
                    all_confidences.iter().sum::<f32>() / all_confidences.len() as f32
                } else {
                    0.0
                };

                println!(
                    "  {:19} │ {:>8.1}% │ {:>5.1}% │ {:>7.1}% │ {:>12.1}%",
                    if model_name.len() > 19 {
                        &model_name[..19]
                    } else {
                        &model_name
                    },
                    avg_precision * 100.0,
                    avg_recall * 100.0,
                    avg_f1 * 100.0,
                    avg_confidence * 100.0
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_analysis() {
        let mut analyzer = ConfidenceAnalyzer::new();

        // Add available models
        analyzer.add_model(
            "GLiNER Small v2.1".to_string(),
            "models/gliner/gliner_small-v2.1/tokenizer.json".to_string(),
            "models/gliner/gliner_small-v2.1/model.onnx".to_string(),
        );

        analyzer.add_model(
            "GLiNER X-Small".to_string(),
            "models/gliner/gliner-x-small/tokenizer.json".to_string(),
            "models/gliner/gliner-x-small/model.onnx".to_string(),
        );

        // Use custom thresholds for faster testing
        analyzer = analyzer.with_thresholds(vec![0.3, 0.5, 0.7]);

        // Set up ground truth data
        analyzer.with_ground_truth_data();

        // Run analysis
        let results = analyzer.analyze_all_models();

        if !results.is_empty() {
            analyzer.print_confidence_analysis(&results);

            // Basic assertions
            assert!(
                results.iter().any(|r| r.threshold == 0.3),
                "Should have results for threshold 0.3"
            );
            assert!(
                results.iter().any(|r| r.threshold == 0.5),
                "Should have results for threshold 0.5"
            );
            assert!(
                results.iter().any(|r| r.threshold == 0.7),
                "Should have results for threshold 0.7"
            );
        } else {
            println!(
                "⚠️  No models available for confidence analysis. Run 'make download-models' to download GLiNER models."
            );
        }
    }
}
