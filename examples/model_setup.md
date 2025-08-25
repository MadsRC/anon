# GLiNER Model Setup Guide

This guide explains how to set up GLiNER models for use with the anonymization SDK.

## Quick Start (Recommended)

The easiest way to get started is using our Makefile:

```bash
# Download models and setup everything
make install

# Or just download models
make models

# Run GLiNER demo
make demo-ner
```

## Model Downloads

### Option 1: Automated Download (Recommended)
Use the Makefile for automatic model management:

```bash
# Download small model (~580MB, recommended for most use cases)
make models-small

# Download large model (~1.4GB, better accuracy)  
make models-large

# Verify model integrity
make verify-models

# Clean up models if needed
make clean-models
```

### Option 2: Manual Download
Download pre-trained GLiNER models from Hugging Face:

```bash
# Create models directory
mkdir models

# Download GLiNER Small (recommended for CPU)
wget -O models/tokenizer.json https://huggingface.co/urchade/gliner_small-v2.1/resolve/main/tokenizer.json
wget -O models/model.onnx https://huggingface.co/urchade/gliner_small-v2.1/resolve/main/model.onnx

# Alternative: GLiNER Multitask Large (better accuracy, requires more resources)
# wget -O models/tokenizer.json https://huggingface.co/urchade/gliner_multitask_large-v0.5/resolve/main/tokenizer.json  
# wget -O models/model.onnx https://huggingface.co/urchade/gliner_multitask_large-v0.5/resolve/main/model.onnx
```

### Option 2: Using Hugging Face Hub (Python)
```python
from huggingface_hub import hf_hub_download

# Download tokenizer
hf_hub_download(
    repo_id="urchade/gliner_small-v2.1",
    filename="tokenizer.json",
    local_dir="./models"
)

# Download model
hf_hub_download(
    repo_id="urchade/gliner_small-v2.1", 
    filename="model.onnx",
    local_dir="./models"
)
```

## Usage Examples

### Basic CPU Usage
```rust
use anon_sdk::detection::{EntityDetector, EntityType, ner::GlinerDetector};

let detector = GlinerDetector::new(
    "models/tokenizer.json",
    "models/model.onnx", 
    vec![EntityType::Person, EntityType::Location, EntityType::Organization]
)?;

let entities = detector.detect("John Smith works at Microsoft in Seattle.")?;
```

### GPU Acceleration
```rust
let gpu_detector = GlinerDetector::with_gpu_acceleration(
    "models/tokenizer.json",
    "models/model.onnx",
    vec![EntityType::Person, EntityType::Location]
)?;
```

### Confidence Threshold Tuning
```rust
let detector = GlinerDetector::new(tokenizer_path, model_path, entity_types)?
    .with_confidence_threshold(0.8)?; // Only entities with >80% confidence
```

## Supported Entity Types

GLiNER models typically support these entity types:

- **Standard Types:**
  - `person` - Person names
  - `location` - Geographic locations
  - `organization` - Companies, institutions
  - `event` - Events, conferences
  - `product` - Products, brands
  - `date` - Dates and times
  - `money` - Monetary amounts

- **Custom Types:**
  You can define custom entity types that GLiNER will attempt to detect:
  ```rust
  vec![
      EntityType::Custom("vehicle".to_string()),
      EntityType::Custom("medical_condition".to_string()),
      EntityType::Custom("technology".to_string()),
  ]
  ```

## Performance Considerations

### Model Selection
- **gliner_small-v2.1**: Faster, lower memory usage, good for CPU
- **gliner_multitask_large-v0.5**: Higher accuracy, requires more resources

### Optimization Tips
1. **Batch Processing**: Process multiple texts together when possible
2. **GPU Usage**: Use CUDA for significant speedup on compatible hardware
3. **Confidence Threshold**: Higher thresholds reduce false positives
4. **Entity Type Filtering**: Only specify entity types you actually need

## Integration with Anonymization

```rust
use anon_sdk::detection::ner::GlinerDetector;
use anon_sdk::algorithms::entity_anonymization::{EntityAnonymization, ReplacementStrategy};

// Setup detector
let detector = GlinerDetector::new(
    "models/tokenizer.json", 
    "models/model.onnx",
    vec![EntityType::Person, EntityType::Location, EntityType::Organization]
)?;

// Configure anonymization
let mut anonymizer = EntityAnonymization::new();
anonymizer.add_replacement_strategy(EntityType::Person, ReplacementStrategy::Pseudonymize);
anonymizer.add_replacement_strategy(EntityType::Location, ReplacementStrategy::Generalize("LOCATION".to_string()));
anonymizer.add_replacement_strategy(EntityType::Organization, ReplacementStrategy::Generalize("ORG".to_string()));

// Apply anonymization
let anonymized = anonymizer.anonymize_text(text, &detector)?;
```

## Troubleshooting

### Common Issues
1. **Model not found**: Ensure model files exist at specified paths
2. **ONNX runtime errors**: Install appropriate ONNX runtime for your platform
3. **CUDA errors**: Verify CUDA compatibility and drivers for GPU acceleration
4. **Memory issues**: Use smaller models or reduce batch size

### Dependencies
Make sure you have the NER feature enabled:
```toml
[dependencies]
anon_sdk = { version = "0.1", features = ["ner"] }
```

### Performance Benchmarks
- **CPU (Intel i7)**: ~50-100 documents/second
- **GPU (RTX 3080)**: ~200-500 documents/second  
- **Memory usage**: 200MB-2GB depending on model size

## License and Attribution
GLiNER models are typically released under Apache 2.0 license. Always check the specific model's license on Hugging Face before commercial use.