# Privacy Methods: Current Implementation vs. Planned Features

This document explains the current privacy-preserving capabilities of the anonymization SDK, planned features (K-anonymity and differential privacy), and how they differ in approach and application.

## 🔄 Current Implementation: Text-Based Pseudonymization

### **What We Have:**
Our current system focuses on **entity-level pseudonymization** for unstructured text data:

```rust
// Pattern: "John Smith works at Microsoft"
// Result: "Alex Williams works at [ORG]"
```

**Key Characteristics:**
- **Data Type**: Unstructured text documents
- **Approach**: Entity detection + replacement
- **Consistency**: Deterministic (same seed → same output)
- **Format**: Preserves original text structure
- **Privacy Model**: Identity obfuscation

### **Implementation Details:**

#### **Entity Detection:**
- **Pattern-based**: Regex for emails, phones, SSNs, credit cards
- **ML-based**: GLiNER for persons, organizations, locations
- **Hybrid**: Combines both approaches for comprehensive coverage

#### **Anonymization Strategies:**
- **Pseudonymize**: `"John Smith" → "Alex Williams"` (realistic fakes)
- **Redact**: `"john@company.com" → "****@company.com"` (format-preserving)
- **Generalize**: `"Microsoft" → "[ORG]"` (category labels)
- **Suppress**: `"(555) 123-4567" → "XXX-XXX-XXXX"` (fixed replacements)

#### **Pool-Based Generation:**
```rust
let pools = PseudonymPools::generate_with_seed(42, 1000);
// Generates consistent fake names, emails, organizations
// Same seed always produces same mappings
```

## 📊 Planned Feature: K-Anonymity

### **What It Would Be:**
K-anonymity is a **statistical privacy method** for structured tabular data.

**Goal**: Ensure each record is indistinguishable from at least k-1 other records.

### **Example:**
```
Original Dataset:
| Name        | Age | ZIP   | Disease    |
|-------------|-----|-------|------------|
| John Smith  | 25  | 10001 | Diabetes   |
| Jane Doe    | 26  | 10001 | Diabetes   |
| Bob Johnson | 25  | 10002 | Hypertension |

K=2 Anonymized:
| Age Range | ZIP   | Disease      |
|-----------|-------|--------------|
| 20-30     | 1000* | Diabetes     |
| 20-30     | 1000* | Diabetes     |
| 20-30     | 1000* | Hypertension |
```

### **Implementation Requirements:**

#### **Data Structure:**
```rust
struct Dataset {
    columns: HashMap<String, Vec<String>>,
    quasi_identifiers: Vec<String>,  // Age, ZIP (not Disease)
    sensitive_attributes: Vec<String>, // Disease
}
```

#### **Algorithm Steps:**
1. **Grouping**: Partition records by quasi-identifier values
2. **Suppression**: Remove records from groups with < k members
3. **Generalization**: 
   - Ages: `25, 26, 27` → `20-30`
   - ZIPs: `10001, 10002` → `1000*`

#### **Generalization Hierarchies:**
```rust
enum GeneralizationLevel {
    Age(AgeRange),      // 25 → 20-30 → 0-50 → *
    Geography(GeoLevel), // 10001 → 1000* → NY → USA → *
}
```

### **Benefits:**
- **Statistical Privacy**: Groups provide plausible deniability
- **Utility Preservation**: Maintains data for analysis
- **Formal Guarantees**: Mathematical privacy definition

### **Challenges:**
- **Data Structure**: Requires tabular format (not free text)
- **Information Loss**: Generalization reduces data utility
- **Curse of Dimensionality**: More quasi-identifiers = more groups = more suppression

## 🔒 Planned Feature: Differential Privacy

### **What It Would Be:**
Differential privacy provides **mathematical guarantees** that individual records cannot be distinguished in query results.

**Goal**: Add calibrated noise so no individual can be identified from statistical queries.

### **Example:**
```rust
// Original query: "How many people have diabetes?"
// Answer: 1,247 people

// DP query with ε=1.0:
// Answer: 1,247 + Laplace_noise(sensitivity/ε) = 1,251 people
```

### **Implementation Requirements:**

#### **Privacy Budget:**
```rust
struct DifferentialPrivacy {
    epsilon: f64,        // Privacy budget (lower = more private)
    delta: f64,          // Failure probability
    sensitivity: f64,    // Query sensitivity (max change from 1 record)
}
```

#### **Noise Mechanisms:**
```rust
impl DifferentialPrivacy {
    fn add_laplace_noise(&self, value: f64) -> f64 {
        let scale = self.sensitivity / self.epsilon;
        value + sample_laplace(scale)
    }
    
    fn add_gaussian_noise(&self, value: f64) -> f64 {
        let sigma = self.sensitivity * sqrt(2 * ln(1.25/self.delta)) / self.epsilon;
        value + sample_gaussian(0.0, sigma)
    }
}
```

#### **Query Types:**
- **Count queries**: "How many records match criteria?"
- **Sum queries**: "What's the total of column X?"
- **Histogram queries**: "Distribution of ages?"

### **Benefits:**
- **Formal Guarantees**: Mathematical privacy proof
- **Composability**: Multiple queries with budget tracking
- **Robust**: Protects against arbitrary auxiliary information

### **Challenges:**
- **Accuracy Trade-off**: More privacy = more noise = less accuracy
- **Budget Depletion**: Each query consumes privacy budget
- **Complex Implementation**: Requires careful sensitivity analysis

## 🆚 Key Differences: Text vs. Tabular Data

### **Current Approach (Text-Based Pseudonymization)**

| Aspect | Details |
|--------|---------|
| **Data Format** | Unstructured text documents |
| **Privacy Model** | Entity-level identity protection |
| **Consistency** | Deterministic mappings (same seed = same result) |
| **Granularity** | Individual entities within text |
| **Use Case** | LLM preprocessing, document sanitization |
| **Restoration** | Possible with mapping tables |

**Example Input:**
```
"John Smith (john@company.com) called from Microsoft's Seattle office."
```

**Example Output:**
```
"Alex Williams (user123@demo.com) called from [ORG]'s [LOCATION] office."
```

### **K-Anonymity (Tabular Data)**

| Aspect | Details |
|--------|---------|
| **Data Format** | Structured tables with rows/columns |
| **Privacy Model** | Group-based statistical protection |
| **Consistency** | Deterministic grouping rules |
| **Granularity** | Record-level within groups |
| **Use Case** | Database publishing, statistical analysis |
| **Restoration** | Impossible (information permanently lost) |

**Example Input:**
```csv
Name,Age,ZIP,Salary
John,25,10001,50000
Jane,26,10002,52000
```

**Example Output:**
```csv
Age_Range,ZIP_Prefix,Salary_Range
20-30,1000*,50000-60000
20-30,1000*,50000-60000
```

### **Differential Privacy (Query Results)**

| Aspect | Details |
|--------|---------|
| **Data Format** | Aggregate statistics/query results |
| **Privacy Model** | Mathematical indistinguishability |
| **Consistency** | Randomized (same query = different noise) |
| **Granularity** | Query-level noise addition |
| **Use Case** | Statistical databases, research data |
| **Restoration** | Not applicable (operates on aggregates) |

**Example Query:**
```sql
SELECT COUNT(*) FROM employees WHERE department='Engineering'
```

**Example Results:**
```
True answer: 42
DP answer (ε=1.0): 42 + noise = 45
DP answer (ε=1.0): 42 + noise = 39  # Different noise each time
```

## 🔄 Compatibility and Conflicts

### **Complementary Use Cases**

These methods can work together for different parts of a privacy pipeline:

```rust
// 1. Text pseudonymization for document processing
let anonymized_docs = text_anonymizer.anonymize(documents);

// 2. Extract structured data from anonymized text
let structured_data = extract_tabular_data(anonymized_docs);

// 3. Apply k-anonymity for further statistical privacy
let k_anon_data = k_anonymity.anonymize(structured_data, k=5);

// 4. Add differential privacy for public queries
let dp_results = dp_mechanism.query(k_anon_data, "SELECT COUNT(*)");
```

### **Fundamental Conflicts**

**Deterministic vs. Probabilistic:**
- **Text pseudonymization**: Requires consistency (same input = same output)
- **Differential privacy**: Requires randomness (same query ≠ same result)

**Data Granularity:**
- **Entity-level**: Individual names, emails within text
- **Record-level**: Entire database rows
- **Query-level**: Aggregate statistics

**Privacy Guarantees:**
- **Pseudonymization**: Identity protection (can be reversed)
- **K-anonymity**: Group indistinguishability (irreversible)
- **Differential privacy**: Mathematical indistinguishability (provable)

## 🎯 Implementation Roadmap

### **Phase 1: Enhanced Text Processing (Current)**
- ✅ Pattern-based entity detection
- ✅ ML-powered entity recognition
- ✅ Format-preserving pseudonymization
- ✅ Pool-based consistent generation

### **Phase 2: Tabular Data Support (Planned)**
```rust
// New data structures
struct TabularDataset { ... }
struct KAnonymityConfig { k: usize, quasi_identifiers: Vec<String> }

// New algorithms
impl KAnonymity {
    fn anonymize(&self, dataset: &TabularDataset) -> Result<TabularDataset>;
}
```

### **Phase 3: Statistical Privacy (Planned)**
```rust
// Query interface
struct PrivacyBudget { epsilon: f64, remaining: f64 }
struct DPQueryEngine { budget: PrivacyBudget }

impl DPQueryEngine {
    fn count_query(&mut self, predicate: Predicate) -> Result<f64>;
    fn histogram_query(&mut self, column: &str) -> Result<Vec<(String, f64)>>;
}
```

## 💡 Recommendations

### **When to Use Each Method:**

**Current Text Pseudonymization:**
- ✅ LLM preprocessing pipelines
- ✅ Document sanitization
- ✅ Development/testing with realistic data
- ✅ Format preservation requirements

**Future K-Anonymity:**
- 📊 Database publishing
- 📊 Research dataset sharing
- 📊 Compliance with group privacy regulations
- 📊 Statistical analysis preservation

**Future Differential Privacy:**
- 📈 Public statistical APIs
- 📈 Research query systems
- 📈 Strong mathematical privacy needs
- 📈 Adversarial privacy scenarios

### **Architecture Decision:**
Keep these as **separate, composable modules** rather than trying to unify them, since they serve different use cases with different data types and privacy models.

## 📚 References

- **K-Anonymity**: Sweeney, L. "k-anonymity: A model for protecting privacy" (2002)
- **Differential Privacy**: Dwork, C. "Differential Privacy: A Survey of Results" (2008)  
- **Text Anonymization**: Dernoncourt, F. "De-identification of patient notes with recurrent neural networks" (2017)
- **GLiNER**: Zaratiana, U. "GLiNER: Generalist Model for Named Entity Recognition" (2023)