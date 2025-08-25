# Model Evaluation Framework

## Overview

This document describes the comprehensive evaluation framework we built for assessing GLiNER model performance. The framework provides automated testing across multiple dimensions: accuracy, performance, confidence analysis, and quantization impact.

## Architecture

### Framework Components

```
tests/
├── model_evaluation.rs          # Core accuracy evaluation
├── performance_benchmarks.rs    # Speed and throughput testing
├── confidence_analysis.rs       # Threshold optimization
├── model_comparison.rs          # Cross-model comparison
├── quantized_model_evaluation.rs # Quantization impact analysis
└── comprehensive_model_evaluation.rs # Unified evaluation runner
```

### Key Abstractions

#### `ModelConfig`
Represents a model configuration for testing:
```rust
pub struct ModelConfig {
    pub name: String,
    pub tokenizer_path: String,
    pub model_path: String,
}
```

#### `TestCase` & `ExpectedEntity`
Defines ground truth for accuracy evaluation:
```rust
pub struct TestCase {
    pub name: String,
    pub text: String,
    pub expected_entities: Vec<ExpectedEntity>,
}

pub struct ExpectedEntity {
    pub entity_type: EntityType,
    pub text: String,
    pub should_detect: bool,
}
```

#### `EvaluationResult`
Captures comprehensive evaluation metrics:
```rust
pub struct EvaluationResult {
    pub model_name: String,
    pub precision: f32,
    pub recall: f32,
    pub f1_score: f32,
    pub processing_time_ms: u128,
    pub confidence_stats: ConfidenceStats,
}
```

## Evaluation Dimensions

### 1. Accuracy Assessment (`model_evaluation.rs`)

**Purpose**: Measure entity detection accuracy using precision, recall, and F1 scores.

**Test Cases**:
- Simple entity detection (person, organization, location)
- Email and phone number detection
- Multiple entities in complex sentences
- Geographic location variations
- Mixed context scenarios

**Metrics**:
- Precision: `true_positives / (true_positives + false_positives)`
- Recall: `true_positives / (true_positives + false_negatives)`
- F1 Score: `2 * (precision * recall) / (precision + recall)`

**Ground Truth Matching**:
- Flexible text matching (exact, substring, case-insensitive)
- Position overlap detection
- Entity type validation

### 2. Performance Benchmarking (`performance_benchmarks.rs`)

**Purpose**: Measure processing speed, latency, and scalability.

**Test Types**:
- **Throughput Test**: Characters/second and entities/second
- **Latency Test**: Response time by text size categories
- **Scale Test**: Performance across different batch sizes

**Text Categories**:
- Small: < 100 characters
- Medium: 100-500 characters  
- Large: > 500 characters

**Metrics**:
- Characters per second
- Entities detected per second
- Average, minimum, maximum latency
- Latency distribution by text size

**Test Data**:
- Synthetic text generation with entity templates
- Real-world document excerpts
- Stress test scenarios (dense entities, long texts)

### 3. Confidence Analysis (`confidence_analysis.rs`)

**Purpose**: Find optimal confidence thresholds and analyze prediction certainty.

**Threshold Range**: 0.1 to 0.9 in 0.1 increments

**Analysis**:
- Precision-recall curves across thresholds
- Optimal threshold identification (highest F1)
- Cross-model threshold comparison
- Confidence distribution statistics

**Ground Truth Dataset**:
- Carefully annotated test cases with exact entity positions
- Challenging cases (hyphenated names, complex emails)
- Edge cases and ambiguous entities

### 4. Cross-Model Comparison (`model_comparison.rs`)

**Purpose**: Direct comparison of multiple models across all dimensions.

**Comparison Types**:
- Accuracy comparison with identical test cases
- Performance head-to-head benchmarking
- Confidence threshold analysis
- Winner identification by category

**Reporting**:
- Detailed performance tables
- Ranking by different criteria
- Best model identification per use case

### 5. Quantization Impact Analysis (`quantized_model_evaluation.rs`)

**Purpose**: Assess accuracy vs. performance trade-offs from quantization.

**Analysis**:
- Accuracy loss calculation: `(full_f1 - quantized_f1) / full_f1 * 100`
- Performance gain measurement: `(quantized_speed - full_speed) / full_speed * 100`
- Model size comparison
- Quantization quality assessment

**Recommendations**:
- Automatic recommendation generation based on trade-offs
- Use case specific advice
- Cost-benefit analysis

## Test Data Design

### Synthetic Text Generation

```rust
let templates = vec![
    "Contact {person} at {org} via {email} or call {phone}.",
    "{person} from {org} will be presenting in {location} next week.",
    // ... more templates
];

// Entity pools for substitution
let people = vec!["John Smith", "Sarah Johnson", ...];
let orgs = vec!["Microsoft", "Google", ...];
```

### Real-World Test Cases

Carefully selected excerpts representing:
- Corporate communications
- Academic papers
- News articles
- Legal documents
- Technical documentation

### Stress Test Scenarios

- **Dense Entity Text**: Maximum entities per character
- **Long Documents**: Multi-paragraph processing
- **Edge Cases**: Hyphenated names, special characters
- **Ambiguous Cases**: Names that could be organizations

## Evaluation Methodology

### Test Execution Flow

1. **Model Loading**: Initialize detector with specified configuration
2. **Confidence Setting**: Apply threshold if specified
3. **Text Processing**: Run detection on test cases
4. **Result Collection**: Capture entities, timing, confidence scores
5. **Metric Calculation**: Compute precision, recall, F1
6. **Report Generation**: Format and display results

### Ground Truth Validation

```rust
fn entities_match(detected: &DetectedEntity, expected: &GroundTruthEntity) -> bool {
    // 1. Entity type must match exactly
    if detected.entity_type != expected.entity_type {
        return false;
    }
    
    // 2. Check position overlap
    let overlaps = detected.start < expected.end && expected.start < detected.end;
    
    // 3. Validate text content similarity
    if overlaps {
        text_similarity_check(detected.text, expected.text)
    } else {
        false
    }
}
```

### Statistical Analysis

- **Confidence Intervals**: Bootstrap sampling for metric uncertainty
- **Significance Testing**: Compare model performance differences
- **Distribution Analysis**: Latency and confidence distributions
- **Correlation Analysis**: Relationship between confidence and accuracy

## Framework Usage

### Running Evaluations

```bash
# Full model comparison
make compare-models

# Individual test suites
cargo test test_compare_gliner_models -- --nocapture
cargo test test_performance_comparison -- --nocapture
cargo test test_confidence_threshold_comparison -- --nocapture
cargo test test_all_quantized_models_accuracy -- --nocapture
```

### Adding New Models

```rust
// In test files, add new model configuration
evaluator.add_model(ModelConfig {
    name: "New Model".to_string(),
    tokenizer_path: "path/to/tokenizer.json".to_string(),
    model_path: "path/to/model.onnx".to_string(),
});
```

### Adding New Test Cases

```rust
evaluator.add_test_case(TestCase {
    name: "Custom Test".to_string(),
    text: "Test text with entities".to_string(),
    expected_entities: vec![
        ExpectedEntity {
            entity_type: EntityType::Person,
            text: "entity text".to_string(),
            should_detect: true,
        },
    ],
});
```

## Output and Reporting

### Console Output Format

```
🔍 MODEL EVALUATION SUMMARY
============================

📊 Model Name Results:
──────────────────────────────────────────────────
  📈 Average Precision: XX.X%
  📈 Average Recall:    XX.X%  
  📈 Average F1 Score:  XX.X%
  ⚡ Average Time:      XXms
  🎯 Average Confidence: XX.X%

  📋 Test Case Results:
    Test Case 1 - P:XX.X% R:XX.X% F1:XX.X% (XXms)
    Test Case 2 - P:XX.X% R:XX.X% F1:XX.X% (XXms)
```

### Detailed Metrics

- Per-test-case breakdown
- Confidence distribution statistics
- Performance percentiles
- Error analysis (false positives/negatives)

### Comparative Analysis

- Side-by-side model comparison tables
- Winner identification by category
- Trade-off analysis
- Recommendation generation

## Framework Benefits

### Automated Testing
- Consistent evaluation across models
- Reproducible results
- Regression detection
- Continuous integration ready

### Comprehensive Coverage
- Multiple evaluation dimensions
- Diverse test scenarios
- Edge case handling
- Real-world applicability

### Decision Support
- Clear performance trade-offs
- Use case specific recommendations
- Confidence in model selection
- Quantitative comparison data

## Best Practices

### Test Case Design
1. **Representative**: Cover real-world usage patterns
2. **Challenging**: Include edge cases and difficult scenarios
3. **Balanced**: Equal representation across entity types
4. **Annotated**: Precise ground truth labeling

### Evaluation Process
1. **Isolation**: Test one variable at a time
2. **Repetition**: Multiple runs for statistical significance
3. **Documentation**: Record all parameters and configurations
4. **Validation**: Cross-check results with manual inspection

### Result Interpretation
1. **Context**: Consider use case requirements
2. **Trade-offs**: Balance accuracy vs. performance
3. **Confidence**: Account for measurement uncertainty
4. **Trends**: Look for patterns across test cases

## Future Enhancements

### Framework Extensions
- GPU performance benchmarking
- Memory usage profiling
- Energy consumption measurement
- Ensemble method evaluation

### Test Coverage Expansion
- Domain-specific test cases
- Multilingual evaluation
- Custom entity type testing
- Adversarial examples

### Automation Improvements
- Continuous benchmarking
- Performance regression alerts
- Automated model ranking
- Report generation

This evaluation framework provides a robust foundation for GLiNER model assessment and comparison, enabling data-driven decisions about model selection and optimization.