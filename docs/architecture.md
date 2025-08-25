# Data Anonymization SDK - System Architecture

## Overview

This document describes the **conceptual framework** and **system design** of the data anonymization SDK. The SDK is designed as a privacy-preserving pipeline that can detect, classify, and anonymize sensitive data while maintaining the ability to restore original values when needed.

## Core Design Principles

### 1. **Multi-Modal Detection**
- Combine deterministic pattern matching with probabilistic machine learning
- Achieve high precision on structured data (emails, SSNs) and semantic understanding on unstructured content
- Support extensible entity types for domain-specific requirements

### 2. **Semantic Preservation**
- Maintain data structure and context during anonymization
- Preserve format characteristics (email domains, phone number patterns)
- Enable meaningful analysis of anonymized data

### 3. **Reversible Transformations**
- Support bidirectional data flow (anonymize ↔ restore)
- Use deterministic, seed-based generation for consistent results
- Maintain transformation mappings for audit and restoration

### 4. **Layered Privacy Controls**
- Multiple anonymization strategies per entity type
- Configurable privacy levels and confidence thresholds
- Formal privacy guarantees (k-anonymity, differential privacy)

## System Architecture

### Pipeline Overview
```
Input Data → Detection → Classification → Transformation → Output
    ↑                                          ↓
Restoration ← Mapping Registry ← Inverse Transform
```

### Layer 1: Detection Engine

**Pattern-Based Detector**
- Regex-based detection for structured PII (emails, phones, SSNs, credit cards, IPs)
- 100% confidence matching for well-defined formats
- Fast, deterministic processing

**Machine Learning Detector (GLiNER)**
- Neural Named Entity Recognition for semantic entities
- Detects persons, organizations, locations, custom entity types
- Configurable confidence thresholds (default: 50%)
- Optional GPU acceleration support

**Hybrid Detection Strategy**
```rust
pub trait EntityDetector {
    fn detect(&self, text: &str) -> Result<Vec<DetectedEntity>>;
    fn supported_entities(&self) -> Vec<EntityType>;
}
```

### Layer 2: Classification System

**Entity Types**
- **Structured**: Email, PhoneNumber, SocialSecurityNumber, CreditCard, IpAddress
- **Semantic**: Person, Location, Organization
- **Custom**: Extensible for domain-specific entities (IntellectualProperty, TradeSecret)

**Detection Results**
```rust
pub struct DetectedEntity {
    pub entity_type: EntityType,
    pub text: String,
    pub start: usize,
    pub end: usize,
    pub confidence: f32,
}
```

### Layer 3: Transformation Engine

**Anonymization Strategies**

| Strategy | Use Case | Example |
|----------|----------|---------|
| **Redact** | Format-preserving masking | `john@company.com` → `****@company.com` |
| **Suppress** | Fixed replacement text | `(555) 123-4567` → `XXX-XXX-XXXX` |
| **Generalize** | Category-based replacement | `Microsoft` → `[ORG]` |
| **Pseudonymize** | Realistic fake generation | `John Smith` → `Alex Williams` |

**Privacy Algorithms**
- **K-Anonymity**: Ensures k identical records for quasi-identifiers
- **Differential Privacy**: Adds calibrated noise for statistical privacy
- **Format Preservation**: Maintains structural characteristics

### Layer 4: Transformation Registry *(Future Enhancement)*

**Reversible Mappings**
```rust
pub struct TransformationRegistry {
    seed: u64,                                    // Deterministic generation
    mappings: HashMap<String, String>,            // original → anonymized
    inverse_mappings: HashMap<String, String>,    // anonymized → original
    entity_generators: HashMap<EntityType, Box<dyn Generator>>,
}
```

**Restoration Capability**
- Bidirectional API: `anonymize()` and `restore()` methods
- Persistent storage for transformation mappings
- Collision detection and resolution

## Use Cases & Applications

### Primary: LLM Data Preprocessing
- Anonymize datasets before sending to external LLM APIs
- Preserve semantic meaning for effective model processing  
- Restore original values in model outputs for downstream use

### Secondary: Privacy-Preserving Analytics
- Enable data sharing while protecting individual privacy
- Support compliance with regulations (GDPR, HIPAA, CCPA)
- Facilitate collaborative research on sensitive datasets

### Advanced: Intellectual Property Protection
- Detect and anonymize proprietary information
- Protect trade secrets and competitive advantages
- Enable safe data sharing across organizational boundaries

## Performance Characteristics

### Detection Performance
- **Pattern-based**: ~1,000 documents/second
- **ML-based**: ~50-500 documents/second (model dependent)
- **Memory usage**: 200MB-2GB (varies with model size)

### Scalability Features
- Parallel processing with Rayon
- GPU acceleration support for ML models
- Streaming processing for large datasets

## Technical Implementation

### Core Technologies
- **Language**: Rust (performance, safety, concurrency)
- **ML Framework**: GLiNER via ONNX Runtime
- **Regex Engine**: High-performance pattern matching
- **Serialization**: Serde for configuration and data exchange

### Key Dependencies
```toml
[dependencies]
rayon = "1.8"           # Parallel processing
ndarray = "0.15"        # Numerical computations  
regex = "1.11"          # Pattern matching
serde = "1.0"           # Serialization
gline-rs = "1"          # ML-based NER (optional)
```

## Extensibility Points

### Custom Entity Types
```rust
EntityType::Custom("company_ip".to_string())
EntityType::Custom("trade_secret".to_string())
```

### Custom Anonymization Strategies
```rust
pub trait AnonymizationAlgorithm {
    fn anonymize(&self, dataset: &Dataset) -> Result<Dataset>;
    fn validate_parameters(&self) -> Result<()>;
}
```

### Custom Detectors
```rust
pub trait EntityDetector {
    fn detect(&self, text: &str) -> Result<Vec<DetectedEntity>>;
    fn supported_entities(&self) -> Vec<EntityType>;
}
```

## Future Enhancements

### Planned Features
1. **Reversible Transformation System**: Seed-based deterministic anonymization
2. **Advanced IP Detection**: Context-aware detection of proprietary information
3. **Policy Engine**: Rule-based anonymization policies
4. **Audit & Compliance**: Detailed logging and compliance reporting
5. **Streaming API**: Real-time processing for large data flows

### Research Areas
- **Contextual Analysis**: Paragraph-level semantic understanding
- **Federated Privacy**: Distributed anonymization across organizations  
- **Adaptive Thresholds**: Dynamic confidence adjustment based on data sensitivity

## Conclusion

The data anonymization SDK provides a robust, extensible framework for privacy-preserving data processing. Its multi-layered architecture balances performance, accuracy, and privacy protection while maintaining the flexibility needed for diverse use cases. The planned addition of reversible transformations will make it particularly suitable for LLM preprocessing workflows where data restoration is critical.