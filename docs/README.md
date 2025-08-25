# Documentation

This directory contains comprehensive documentation for the Anonymization SDK.

## Model Evaluation & Comparison

- **[GLiNER Model Comparison](gliner_model_comparison.md)** - Comprehensive comparison of all GLiNER models with performance benchmarks and recommendations
- **[Model Evaluation Framework](model_evaluation_framework.md)** - Technical documentation of our automated evaluation system
- **[GLiNER Model Setup](gliner_model_setup.md)** - Basic model setup and installation guide

## System Architecture

- **[Architecture Overview](architecture.md)** - High-level system architecture and components
- **[Pool System Architecture](pool_system_architecture.md)** - Detailed pool management system design

## Features

- **[Deterministic Pseudonymization](deterministic_pseudonymization.md)** - Consistent entity replacement system
- **[CLI Pool Management](cli_pool_management.md)** - Command-line interface for managing entity pools

## Quick Start

1. **Model Setup**: Follow [GLiNER Model Setup](gliner_model_setup.md) to install models
2. **Model Selection**: Review [GLiNER Model Comparison](gliner_model_comparison.md) to choose optimal model
3. **Performance Testing**: Use `make compare-models` to run comprehensive evaluations

## Key Findings Summary

### Recommended Model Configurations

| Use Case | Model | Configuration | Performance |
|----------|-------|---------------|-------------|
| **Maximum Accuracy** | GLiNER X-Small (Full) | Default threshold 0.3 | 100% F1, 42ms latency |
| **Balanced (Recommended)** | GLiNER X-Small (Quantized) | Default threshold 0.3 | 71.7% F1, 16ms latency |
| **Maximum Performance** | GLiNER Small v2.1 (Quantized) + Hybrid | Hybrid detection | 62.5% F1, 18ms latency |

### Critical Insights

- ✅ **GLiNER X-Small models solve email detection issues** (100% vs 0% for Small v2.1)
- 🚀 **Quantized models provide 2-2.5x performance improvements** with acceptable accuracy loss
- 🎯 **Optimal confidence threshold is 0.3** for all models
- ⚖️ **X-Small Quantized offers best balance** for production use

## Running Evaluations

```bash
# Download all models
make download-models

# Run comprehensive model comparison
make compare-models

# Individual evaluations
cargo test test_compare_gliner_models -- --nocapture
cargo test test_performance_comparison -- --nocapture
```

For detailed technical information, see [Model Evaluation Framework](model_evaluation_framework.md).