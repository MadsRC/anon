# 🔒 Anonymization SDK

A high-performance, safe anonymization library for data privacy protection, combining pattern-based and ML-powered entity detection with flexible anonymization strategies.

## ✨ Features

- 🤖 **Advanced Entity Detection**: GLiNER ML models + regex patterns
- 🎭 **Smart Anonymization**: Pseudonymization, generalization, suppression, redaction
- ⚡ **High Performance**: Rust-powered with GPU acceleration support
- 🛡️ **Privacy-First**: K-anonymity, differential privacy, format preservation
- 🔧 **Easy Integration**: Simple APIs with comprehensive examples

## 🚀 Quick Start

Get up and running in 3 commands:

```bash
# 1. Download GLiNER models and build
make install

# 2. Run pattern-based demo
make demo

# 3. Run ML-powered demo  
make demo-ner
```

### What You'll See

**Pattern Detection:**
```
📧 Emails: john.doe@company.com → ****@company.com
📞 Phones: (555) 123-4567 → XXX-XXX-XXXX  
🆔 SSNs: 123-45-6789 → [SSN]
💳 Cards: 4532-1234-5678-9012 → ****-****-****-9012
```

**ML Detection + Anonymization:**
```
🧠 "John Smith works at Microsoft" 
   → "Alex Williams works at [ORG]"

🎯 Detected: John Smith (person, 99% confidence)
             Microsoft (organization, 98% confidence)
```

## 📦 Installation

### Prerequisites
- **Rust 1.70+**: [Install Rust](https://rustup.rs/)
- **~600MB disk space**: For GLiNER models

### Option 1: Full Setup (Recommended)
```bash
git clone <repository>
cd anon
make install        # Builds everything

# Download and setup models
make download-models
mkdir -p $HOME/.anon
cp -r models $HOME/.anon/

# Binary is now available at: target/release/anon
```

### Option 2: Manual Setup
```bash
# Install dependencies
cargo build

# Download models (optional - for ML features)
make download-models
mkdir -p $HOME/.anon
cp -r models $HOME/.anon/

# Run examples
cargo run --example gliner_demo
```

## 🎮 Usage Examples

### Pattern-Based Detection
```rust
use anon_sdk::detection::patterns::PatternDetector;

let detector = PatternDetector::new()?;
let entities = detector.detect("Contact: john@example.com, (555) 123-4567")?;
// → [Email: "john@example.com", Phone: "(555) 123-4567"]
```

### ML-Powered Detection
```rust  
use anon_sdk::detection::ner::GlinerDetector;
use anon_sdk::detection::EntityType;

let detector = GlinerDetector::new(
    "models/gliner/gliner-x-small/tokenizer.json",
    "models/gliner/gliner-x-small/model.onnx", 
    vec![EntityType::Person, EntityType::Organization]
)?;

let entities = detector.detect("John Smith works at Microsoft")?;
// → [Person: "John Smith" (99%), Organization: "Microsoft" (98%)]
```

### Smart Anonymization
```rust
use anon_sdk::algorithms::entity_anonymization::{EntityAnonymization, ReplacementStrategy};

let mut anonymizer = EntityAnonymization::new();
anonymizer.add_replacement_strategy(EntityType::Person, ReplacementStrategy::Pseudonymize);
anonymizer.add_replacement_strategy(EntityType::Email, ReplacementStrategy::Redact);

let result = anonymizer.anonymize_text(
    "Contact John Smith at john@company.com",
    &detector
)?;
// → "Contact Alex Williams at ****@company.com"
```

### CLI Usage

After installation, use the `anon` binary for command-line anonymization:

```bash
# Use the built binary (from project directory)
echo "Hi Mads, call me at (555) 123-4567" | ./target/release/anon --model-path $HOME/.anon/models/gliner/gliner-x-small

# Or add to PATH for global access
export PATH=$PATH:$(pwd)/target/release
echo "Contact John Smith at john@example.com" | anon --model-path $HOME/.anon/models/gliner/gliner-x-small
```

## 🛠️ Development Commands

Our Makefile provides comprehensive automation:

```bash
# 🤖 Model Management
make download-models           # Download all GLiNER models  
make download-gliner_small-v2.1 # Download GLiNER small v2.1 model
make download-gliner-x-small   # Download GLiNER x-small model

# 🚀 Development  
make build          # Build project
make test           # Run all tests
make check          # Check compilation
make lint           # Run clippy linter  
make format         # Format code

# 📚 Examples & Demos
make demo           # Pattern-based demo
make demo-ner       # GLiNER ML demo
make benchmark      # Performance tests

# 🧹 Cleanup
make clean          # Clean build artifacts
make clean-all      # Clean everything
```

## 🏗️ Architecture

```
anon/
├── src/
│   ├── detection/           # Entity detection systems
│   │   ├── patterns.rs      # Regex-based detection  
│   │   ├── ner.rs           # GLiNER ML detection
│   │   └── hybrid.rs        # Hybrid detection
│   ├── algorithms/          # Anonymization strategies
│   │   ├── k_anonymity.rs   # K-anonymity
│   │   ├── differential_privacy.rs
│   │   ├── entity_anonymization.rs
│   │   ├── generalization.rs
│   │   └── suppression.rs
│   ├── evaluation/          # Model evaluation framework
│   └── utils.rs             # Helper functions
├── models/                  # GLiNER model files
├── examples/                # Usage demonstrations
├── docs/                    # Documentation
└── tests/                   # Comprehensive test suite
```

## 🎯 Entity Detection

### Pattern-Based (Built-in)
- 📧 **Email addresses**: RFC-compliant patterns
- 📞 **Phone numbers**: US/International formats  
- 🆔 **Social Security Numbers**: XXX-XX-XXXX format
- 💳 **Credit cards**: Visa, MasterCard, Amex, Discover
- 🌐 **IP addresses**: IPv4 validation

### ML-Powered (GLiNER)
- 👤 **Persons**: Names with 85-99% accuracy
- 🏢 **Organizations**: Companies, institutions  
- 📍 **Locations**: Cities, countries, addresses
- 📧 **Email addresses**: X-Small models only (100% accuracy)
- 📞 **Phone numbers**: With hybrid detection
- 🎨 **Custom entities**: Define your own types

### 🔍 Model Comparison Results

Our comprehensive evaluation framework tested 4 GLiNER model variants:

| Model | Accuracy (F1) | Speed (chars/sec) | Latency | Recommended Use |
|-------|---------------|------------------|---------|-----------------|
| **X-Small (Full)** | 100.0% | 5,867 | 42ms | Maximum accuracy |
| **X-Small (Quantized)** ⭐ | 71.7% | 14,534 | 16ms | **Balanced (recommended)** |
| Small v2.1 (Quantized) | 62.5% | 13,324 | 18ms | High performance |
| Small v2.1 (Full) | 61.1% | 6,515 | 37ms | Legacy compatibility |

**Key Findings:**
- ✅ **X-Small models solve email detection** (100% vs 0% for Small v2.1)
- 🚀 **Quantized models are 2-2.5x faster** with acceptable accuracy loss
- ⚖️ **X-Small Quantized offers best balance** for production use

Run `make compare-models` for detailed analysis. See [docs/gliner_model_comparison.md](docs/gliner_model_comparison.md) for full results.

## 🎭 Anonymization Strategies

| Strategy | Description | Example |
|----------|-------------|---------|
| **Redact** | Format-preserving masking | `john@example.com` → `****@example.com` |
| **Suppress** | Replace with fixed text | `(555) 123-4567` → `XXX-XXX-XXXX` |
| **Generalize** | Replace with category | `Microsoft` → `[ORG]` |
| **Pseudonymize** | Generate realistic fakes | `John Smith` → `Alex Williams` |

## ⚡ Performance

- **Speed**: ~1000 documents/second (pattern), ~50-500/second (ML)
- **Memory**: ~200MB-2GB depending on model size
- **GPU Support**: CUDA acceleration available
- **Parallel Processing**: Multi-threaded with Rayon

## 🧪 Testing

```bash
# Run all tests
make test

# Run specific test categories
cargo test patterns          # Pattern detection tests  
cargo test ner               # ML detection tests
cargo test algorithms        # Anonymization tests

# Performance benchmarks  
make benchmark
```

## 📝 Examples

Check out comprehensive examples in the [`examples/`](examples/) directory:

- **`ner_demo.rs`**: Pattern-based detection and anonymization
- **`gliner_demo.rs`**: ML-powered entity recognition  
- **`model_setup.md`**: Detailed model setup guide

## 🤝 Contributing

1. **Fork** the repository
2. **Create** a feature branch: `git checkout -b feature/amazing-feature`
3. **Test** your changes: `make test`
4. **Lint** your code: `make lint`
5. **Submit** a pull request

## 📄 License

This project is licensed under a proprietary license. See [LICENSE](LICENSE).

## 🙏 Acknowledgments

- **GLiNER**: For state-of-the-art NER models
- **Rust Community**: For incredible tooling and libraries
- **Hugging Face**: For model hosting and distribution

---

⭐ **Star this repo** if you find it useful!

🔗 **[View Examples](examples/)** | **[Documentation](docs/)**
