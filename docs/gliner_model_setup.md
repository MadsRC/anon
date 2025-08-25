# GLiNER Model Setup Guide

This document explains how to test different GLiNER model sizes for improved email detection.

## Current Model Performance

The current medium GLiNER model shows limitations in email detection:
- ❌ `john.smith@microsoft.com` → breaks into `person: john.smith` + `organization: microsoft`
- ❌ `user@domain.com` → breaks into `person: user` + no organization
- ✅ `jane@company.org` → detected as `email` (67.7% confidence) in some contexts

## Testing Larger Models

### 1. GLiNER Large Model (Recommended)
**Source:** https://huggingface.co/onnx-community/gliner-multitask-large-v0.5

**Easy Setup with Makefile:**
```bash
make models-large
```

**Manual Setup:**
```bash
mkdir -p models/gliner-large
cd models/gliner-large
curl -L -o tokenizer.json "https://huggingface.co/onnx-community/gliner-multitask-large-v0.5/resolve/main/tokenizer.json"
curl -L -o model.onnx "https://huggingface.co/onnx-community/gliner-multitask-large-v0.5/resolve/main/onnx/model.onnx"
```

**Test with:**
```bash
make test-large-model
# OR manually:
cargo test test_gliner_large_model_email_robustness -- --nocapture
./target/debug/anon --detector ner --model-path models/gliner-large --debug
```

### 2. GLiNER X-Large Model (Maximum Capability)
**Source:** https://huggingface.co/knowledgator/gliner-x-large

**Easy Setup with Makefile:**
```bash
make models-xlarge
```

**Manual Setup:**
```bash
mkdir -p models/gliner-xlarge
cd models/gliner-xlarge
curl -L -o tokenizer.json "https://huggingface.co/knowledgator/gliner-x-large/resolve/main/tokenizer.json"
curl -L -o model.onnx "https://huggingface.co/knowledgator/gliner-x-large/resolve/main/pytorch_model.onnx"
```

**Test with:**
```bash
make test-xlarge-model
# OR manually:
./target/debug/anon --detector ner --model-path models/gliner-xlarge --debug
```

## Expected vs Actual Results

**Expected improvements from larger models:**
1. ✅ Better generalization to new entity types like "email"
2. ✅ Higher confidence for email detection vs person/org breakdown
3. ✅ More robust detection across different email formats
4. ✅ Reduced false decomposition of structured data

**Actual test results:**
1. ❌ **Medium model**: Works but breaks emails into person+org components consistently
2. ❌ **Large model**: Architecturally incompatible (different tensor requirements)
3. ❌ **X-Large model**: Poor detection overall (quantization side effects)

**Conclusion**: Current GLiNER models from different sources have compatibility issues and none properly detect emails as single entities.

## Running Comparison Tests

**Easy testing with Makefile:**
```bash
make compare-models              # Compare all available models
make test-email-detection        # Test current model email detection
make test-large-model           # Test large model (downloads if needed)
make test-xlarge-model          # Test X-Large model (downloads if needed)
```

**Manual testing:**
```bash
cargo test test_gliner_model_comparison_for_emails -- --nocapture
cargo test test_gliner_large_model_email_robustness -- --nocapture
cargo test test_email_detection_summary -- --nocapture
```

## CLI Usage

Default (medium model):
```bash
echo "john.smith@microsoft.com" | ./target/debug/anon --detector ner --debug
```

Large model:
```bash
echo "john.smith@microsoft.com" | ./target/debug/anon --detector ner --model-path models/gliner-large --debug
```

## Why Model Size Matters

GLiNER uses **zero-shot entity recognition**, meaning it can detect any entity type you specify without retraining. However:

- **Smaller models** have limited generalization and may decompose structured data
- **Larger models** better understand entity boundaries and context
- **Email detection** requires understanding that email addresses are single entities, not person+organization components

## Fallback Strategy

If larger models don't solve email detection:
1. **Use hybrid detection** (recommended) - combines pattern + NER strengths
2. **Adjust confidence thresholds** for email entities specifically  
3. **Custom post-processing** to merge person@organization patterns back into emails

The hybrid approach remains optimal for production use as it combines:
- **Pattern detection**: 100% reliable for structured data (emails, phones, etc.)
- **NER detection**: Excellent for contextual entities (names, places, etc.)