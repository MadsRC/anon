use crate::detection::{EntityDetector, EntityType, ner::GlinerDetector};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub name: String,
    pub tokenizer_path: String,
    pub model_path: String,
}

#[derive(Debug, Clone)]
pub struct TestCase {
    pub name: String,
    pub text: String,
    pub expected_entities: Vec<ExpectedEntity>,
}

#[derive(Debug, Clone)]
pub struct ExpectedEntity {
    pub entity_type: EntityType,
    pub text: String,
    pub should_detect: bool, // Whether we expect this entity to be detected
}

#[derive(Debug)]
pub struct EvaluationResult {
    pub model_name: String,
    pub test_case_name: String,
    #[allow(dead_code)]
    pub detected_entities: Vec<DetectedEntityResult>,
    pub precision: f32,
    pub recall: f32,
    pub f1_score: f32,
    pub processing_time_ms: u128,
    pub confidence_stats: ConfidenceStats,
}

#[derive(Debug)]
pub struct DetectedEntityResult {
    #[allow(dead_code)]
    pub entity_type: EntityType,
    #[allow(dead_code)]
    pub text: String,
    pub confidence: f32,
    #[allow(dead_code)]
    pub start: usize,
    #[allow(dead_code)]
    pub end: usize,
    pub is_correct: bool, // Whether this detection matches expected entities
}

#[derive(Debug)]
pub struct ConfidenceStats {
    pub mean: f32,
    #[allow(dead_code)]
    pub min: f32,
    #[allow(dead_code)]
    pub max: f32,
    #[allow(dead_code)]
    pub std_dev: f32,
}

impl ModelConfig {
    pub fn gliner_small() -> Self {
        Self {
            name: "GLiNER Small v2.1".to_string(),
            tokenizer_path: "models/gliner/gliner_small-v2.1/tokenizer.json".to_string(),
            model_path: "models/gliner/gliner_small-v2.1/model.onnx".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn gliner_x_small() -> Self {
        Self {
            name: "GLiNER X-Small".to_string(),
            tokenizer_path: "models/gliner/gliner_x_small/tokenizer.json".to_string(),
            model_path: "models/gliner/gliner_x_small/model.onnx".to_string(),
        }
    }
}

pub struct ModelEvaluator {
    test_cases: Vec<TestCase>,
    models: Vec<ModelConfig>,
    entity_types: Vec<EntityType>,
}

impl Default for ModelEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelEvaluator {
    pub fn new() -> Self {
        Self {
            test_cases: Vec::new(),
            models: Vec::new(),
            entity_types: vec![
                EntityType::Person,
                EntityType::Organization,
                EntityType::Location,
                EntityType::Email,
                EntityType::PhoneNumber,
            ],
        }
    }

    pub fn add_model(&mut self, model: ModelConfig) {
        self.models.push(model);
    }

    pub fn add_test_case(&mut self, test_case: TestCase) {
        self.test_cases.push(test_case);
    }

    #[allow(dead_code)]
    pub fn with_entity_types(mut self, entity_types: Vec<EntityType>) -> Self {
        self.entity_types = entity_types;
        self
    }

    pub fn evaluate_all_models(&self, confidence_threshold: f32) -> Vec<EvaluationResult> {
        let mut results = Vec::new();

        for model in &self.models {
            println!("📊 Evaluating model: {}", model.name);

            let detector_result = GlinerDetector::new(
                &model.tokenizer_path,
                &model.model_path,
                self.entity_types.clone(),
            );

            match detector_result {
                Ok(detector) => {
                    let mut detector = detector
                        .with_confidence_threshold(confidence_threshold)
                        .unwrap_or_else(|_| {
                            // If threshold setting fails, create a new detector
                            GlinerDetector::new(
                                &model.tokenizer_path,
                                &model.model_path,
                                self.entity_types.clone(),
                            )
                            .unwrap()
                        });

                    for test_case in &self.test_cases {
                        let result =
                            self.evaluate_single_case(&model.name, &mut detector, test_case);
                        results.push(result);
                    }
                }
                Err(e) => {
                    println!("❌ Failed to load model {}: {}", model.name, e);
                }
            }
        }

        results
    }

    fn evaluate_single_case(
        &self,
        model_name: &str,
        detector: &mut GlinerDetector,
        test_case: &TestCase,
    ) -> EvaluationResult {
        let start_time = Instant::now();

        let detected_entities = detector.detect(&test_case.text).unwrap_or_default();

        let processing_time = start_time.elapsed().as_millis();

        // Convert to DetectedEntityResult and evaluate correctness
        let mut detected_results = Vec::new();
        for entity in &detected_entities {
            let is_correct = self.is_detection_correct(entity, &test_case.expected_entities);
            detected_results.push(DetectedEntityResult {
                entity_type: entity.entity_type.clone(),
                text: entity.text.clone(),
                confidence: entity.confidence,
                start: entity.start,
                end: entity.end,
                is_correct,
            });
        }

        // Calculate precision, recall, F1
        let true_positives = detected_results.iter().filter(|e| e.is_correct).count() as f32;
        let false_positives = detected_results.iter().filter(|e| !e.is_correct).count() as f32;
        let expected_positive_count = test_case
            .expected_entities
            .iter()
            .filter(|e| e.should_detect)
            .count() as f32;
        let _false_negatives = expected_positive_count - true_positives;

        let precision = if (true_positives + false_positives) > 0.0 {
            true_positives / (true_positives + false_positives)
        } else {
            0.0
        };

        let recall = if expected_positive_count > 0.0 {
            true_positives / expected_positive_count
        } else {
            0.0
        };

        let f1_score = if (precision + recall) > 0.0 {
            2.0 * (precision * recall) / (precision + recall)
        } else {
            0.0
        };

        // Calculate confidence statistics
        let confidence_stats = self.calculate_confidence_stats(&detected_results);

        EvaluationResult {
            model_name: model_name.to_string(),
            test_case_name: test_case.name.clone(),
            detected_entities: detected_results,
            precision,
            recall,
            f1_score,
            processing_time_ms: processing_time,
            confidence_stats,
        }
    }

    fn is_detection_correct(
        &self,
        detected: &crate::detection::DetectedEntity,
        expected: &[ExpectedEntity],
    ) -> bool {
        expected.iter().any(|exp| {
            exp.should_detect
                && exp.entity_type == detected.entity_type
                && self.text_matches(&exp.text, &detected.text)
        })
    }

    fn text_matches(&self, expected: &str, detected: &str) -> bool {
        // Allow for partial matches and case insensitivity
        let exp_lower = expected.to_lowercase();
        let det_lower = detected.to_lowercase();

        // Exact match
        if exp_lower == det_lower {
            return true;
        }

        // Contains match (detected contains expected or vice versa)
        exp_lower.contains(&det_lower) || det_lower.contains(&exp_lower)
    }

    fn calculate_confidence_stats(&self, detected: &[DetectedEntityResult]) -> ConfidenceStats {
        if detected.is_empty() {
            return ConfidenceStats {
                mean: 0.0,
                min: 0.0,
                max: 0.0,
                std_dev: 0.0,
            };
        }

        let confidences: Vec<f32> = detected.iter().map(|e| e.confidence).collect();
        let mean = confidences.iter().sum::<f32>() / confidences.len() as f32;
        let min = confidences.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max = confidences.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

        let variance =
            confidences.iter().map(|c| (c - mean).powi(2)).sum::<f32>() / confidences.len() as f32;
        let std_dev = variance.sqrt();

        ConfidenceStats {
            mean,
            min,
            max,
            std_dev,
        }
    }

    pub fn print_summary_report(&self, results: &[EvaluationResult]) {
        println!("\n🔍 MODEL EVALUATION SUMMARY");
        println!("============================");

        // Group results by model
        let mut model_results: HashMap<String, Vec<&EvaluationResult>> = HashMap::new();
        for result in results {
            model_results
                .entry(result.model_name.clone())
                .or_default()
                .push(result);
        }

        for (model_name, model_results) in model_results {
            println!("\n📊 {} Results:", model_name);
            println!("{}", "─".repeat(50));

            let avg_precision =
                model_results.iter().map(|r| r.precision).sum::<f32>() / model_results.len() as f32;
            let avg_recall =
                model_results.iter().map(|r| r.recall).sum::<f32>() / model_results.len() as f32;
            let avg_f1 =
                model_results.iter().map(|r| r.f1_score).sum::<f32>() / model_results.len() as f32;
            let avg_time = model_results
                .iter()
                .map(|r| r.processing_time_ms)
                .sum::<u128>()
                / model_results.len() as u128;
            let avg_confidence = model_results
                .iter()
                .map(|r| r.confidence_stats.mean)
                .sum::<f32>()
                / model_results.len() as f32;

            println!("  📈 Average Precision: {:.2}%", avg_precision * 100.0);
            println!("  📈 Average Recall:    {:.2}%", avg_recall * 100.0);
            println!("  📈 Average F1 Score:  {:.2}%", avg_f1 * 100.0);
            println!("  ⚡ Average Time:      {}ms", avg_time);
            println!("  🎯 Average Confidence: {:.2}%", avg_confidence * 100.0);

            println!("\n  📋 Test Case Results:");
            for result in &model_results {
                println!(
                    "    {} - P:{:.1}% R:{:.1}% F1:{:.1}% ({}ms)",
                    result.test_case_name,
                    result.precision * 100.0,
                    result.recall * 100.0,
                    result.f1_score * 100.0,
                    result.processing_time_ms
                );
            }
        }
    }
}

// Standard test cases for comprehensive evaluation
pub fn create_standard_test_cases() -> Vec<TestCase> {
    vec![
        TestCase {
            name: "Simple Entities".to_string(),
            text: "John Smith works at Microsoft in Seattle.".to_string(),
            expected_entities: vec![
                ExpectedEntity { entity_type: EntityType::Person, text: "John Smith".to_string(), should_detect: true },
                ExpectedEntity { entity_type: EntityType::Organization, text: "Microsoft".to_string(), should_detect: true },
                ExpectedEntity { entity_type: EntityType::Location, text: "Seattle".to_string(), should_detect: true },
            ],
        },
        TestCase {
            name: "Email and Phone".to_string(),
            text: "Contact jane.doe@company.org at (555) 123-4567 for more information.".to_string(),
            expected_entities: vec![
                ExpectedEntity { entity_type: EntityType::Person, text: "jane.doe".to_string(), should_detect: false }, // May be detected as person
                ExpectedEntity { entity_type: EntityType::Email, text: "jane.doe@company.org".to_string(), should_detect: true },
                ExpectedEntity { entity_type: EntityType::PhoneNumber, text: "(555) 123-4567".to_string(), should_detect: true },
                ExpectedEntity { entity_type: EntityType::Organization, text: "company".to_string(), should_detect: false }, // May be detected from email
            ],
        },
        TestCase {
            name: "Multiple People and Orgs".to_string(),
            text: "The meeting between Alice Johnson from Google and Bob Wilson from Amazon is scheduled for tomorrow.".to_string(),
            expected_entities: vec![
                ExpectedEntity { entity_type: EntityType::Person, text: "Alice Johnson".to_string(), should_detect: true },
                ExpectedEntity { entity_type: EntityType::Person, text: "Bob Wilson".to_string(), should_detect: true },
                ExpectedEntity { entity_type: EntityType::Organization, text: "Google".to_string(), should_detect: true },
                ExpectedEntity { entity_type: EntityType::Organization, text: "Amazon".to_string(), should_detect: true },
            ],
        },
        TestCase {
            name: "Complex Email Case".to_string(),
            text: "Send reports to admin@test-domain.co.uk and backup@company.org immediately.".to_string(),
            expected_entities: vec![
                ExpectedEntity { entity_type: EntityType::Email, text: "admin@test-domain.co.uk".to_string(), should_detect: true },
                ExpectedEntity { entity_type: EntityType::Email, text: "backup@company.org".to_string(), should_detect: true },
            ],
        },
        TestCase {
            name: "Geographic Locations".to_string(),
            text: "The conference will be held in New York City, with satellite events in London and Tokyo.".to_string(),
            expected_entities: vec![
                ExpectedEntity { entity_type: EntityType::Location, text: "New York City".to_string(), should_detect: true },
                ExpectedEntity { entity_type: EntityType::Location, text: "London".to_string(), should_detect: true },
                ExpectedEntity { entity_type: EntityType::Location, text: "Tokyo".to_string(), should_detect: true },
            ],
        },
        TestCase {
            name: "Mixed Context".to_string(),
            text: "Dr. Sarah Chen from Stanford University will present her research on AI at the San Francisco conference. Contact her at s.chen@stanford.edu or call (650) 555-0123.".to_string(),
            expected_entities: vec![
                ExpectedEntity { entity_type: EntityType::Person, text: "Sarah Chen".to_string(), should_detect: true },
                ExpectedEntity { entity_type: EntityType::Organization, text: "Stanford University".to_string(), should_detect: true },
                ExpectedEntity { entity_type: EntityType::Location, text: "San Francisco".to_string(), should_detect: true },
                ExpectedEntity { entity_type: EntityType::Email, text: "s.chen@stanford.edu".to_string(), should_detect: true },
                ExpectedEntity { entity_type: EntityType::PhoneNumber, text: "(650) 555-0123".to_string(), should_detect: true },
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_evaluator_framework() {
        let mut evaluator = ModelEvaluator::new();

        // Add available models
        evaluator.add_model(ModelConfig::gliner_small());

        // Add test cases
        for test_case in create_standard_test_cases() {
            evaluator.add_test_case(test_case);
        }

        // Run evaluation with different confidence thresholds
        let confidence_thresholds = vec![0.3, 0.5, 0.7];

        for threshold in confidence_thresholds {
            println!("\n🔍 Evaluating with confidence threshold: {}", threshold);
            let results = evaluator.evaluate_all_models(threshold);

            if !results.is_empty() {
                evaluator.print_summary_report(&results);
            } else {
                println!(
                    "⚠️  No models available for evaluation. Run 'make download-models' to download GLiNER models."
                );
            }
        }
    }
}
