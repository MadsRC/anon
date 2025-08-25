# Proposed EntityTypes for Anonymization SDK

This document outlines proposed new EntityTypes that could be implemented in the anonymization SDK to expand its data detection and protection capabilities.

## Current EntityTypes Analysis

### Pattern-based Detection
- ✅ Email
- ✅ PhoneNumber  
- ✅ SocialSecurityNumber
- ✅ CreditCard
- ✅ IpAddress

### ML-based Detection (GLiNER)
- ✅ Person
- ✅ Location
- ✅ Organization
- ✅ Email (with X-Small models)
- ✅ Custom entities (product, event)

## Proposed New EntityTypes

### 1. Financial Identifiers

#### `BankAccountNumber`
**Priority**: High  
**Detection Method**: Pattern-based  
**Rationale**: Critical financial PII commonly found in documents  
**Pattern Examples**:
- US: 6-17 digit account numbers
- IBAN: International format (e.g., GB29 NWBK 6016 1331 9268 19)
- Routing numbers: 9-digit ABA routing numbers

**Implementation Notes**:
- Regex patterns for different formats
- Luhn algorithm validation where applicable
- Country-specific variants

#### `CreditScore`
**Priority**: Medium  
**Detection Method**: Pattern-based  
**Rationale**: Sensitive financial information  
**Pattern Examples**:
- FICO: 300-850 range
- VantageScore: 300-850 range
- Context-aware detection (preceded by "credit score", "FICO", etc.)

#### `TaxId`
**Priority**: High  
**Detection Method**: Pattern-based  
**Rationale**: Government identification numbers beyond SSN  
**Pattern Examples**:
- EIN (Employer Identification Number): XX-XXXXXXX
- ITIN (Individual Taxpayer Identification Number): XXX-XX-XXXX
- VAT numbers (European): Country-specific formats

### 2. Medical Identifiers

#### `MedicalRecordNumber`
**Priority**: High  
**Detection Method**: Pattern-based + Context  
**Rationale**: HIPAA compliance requirement  
**Pattern Examples**:
- Hospital MRN: Usually 6-10 digits
- Patient ID formats
- Context: "MRN:", "Patient ID:", "Medical Record #"

#### `InsuranceId`
**Priority**: High  
**Detection Method**: Pattern-based  
**Rationale**: Protected health information  
**Pattern Examples**:
- Insurance member IDs: Various formats
- Group numbers
- Policy numbers

#### `DrugId`
**Priority**: Medium  
**Detection Method**: Pattern-based  
**Rationale**: Prescription information privacy  
**Pattern Examples**:
- NDC (National Drug Code): XXXXX-XXXX-XX
- DEA numbers: 2 letters + 7 digits
- Prescription numbers

### 3. Government Identifiers

#### `PassportNumber`
**Priority**: High  
**Detection Method**: Pattern-based  
**Rationale**: International travel document  
**Pattern Examples**:
- US: 9 characters (letters + numbers)
- UK: 9 characters
- Country-specific formats

#### `DriversLicense`
**Priority**: High  
**Detection Method**: Pattern-based  
**Rationale**: Common form of identification  
**Pattern Examples**:
- State-specific formats (US)
- Provincial formats (Canada)
- National formats (other countries)

#### `VoterRegistration`
**Priority**: Medium  
**Detection Method**: Pattern-based  
**Rationale**: Electoral privacy  
**Pattern Examples**:
- State-specific voter ID formats
- Registration numbers

### 4. Digital Identifiers

#### `MacAddress`
**Priority**: Medium  
**Detection Method**: Pattern-based  
**Rationale**: Device identification  
**Pattern Examples**:
- Standard format: XX:XX:XX:XX:XX:XX
- Windows format: XX-XX-XX-XX-XX-XX
- Cisco format: XXXX.XXXX.XXXX

#### `Uuid`
**Priority**: Medium  
**Detection Method**: Pattern-based  
**Rationale**: Unique identifiers in systems  
**Pattern Examples**:
- Standard UUID: XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX
- Variants: with/without hyphens

#### `ApiKey`
**Priority**: High  
**Detection Method**: Pattern-based + Entropy  
**Rationale**: Security credential protection  
**Pattern Examples**:
- AWS keys: AKIA followed by 16 characters
- JWT tokens: Base64 encoded with dots
- Generic API keys: High-entropy strings

#### `CryptocurrencyAddress`
**Priority**: Medium  
**Detection Method**: Pattern-based  
**Rationale**: Financial privacy  
**Pattern Examples**:
- Bitcoin: Base58 encoding, starts with 1, 3, or bc1
- Ethereum: 0x followed by 40 hex characters
- Other cryptocurrencies

### 5. Biometric Identifiers

#### `BiometricTemplate`
**Priority**: High  
**Detection Method**: Pattern-based + Context  
**Rationale**: Highly sensitive biometric data  
**Pattern Examples**:
- Fingerprint minutiae data
- Facial recognition templates
- Voice print identifiers
- Context detection: "fingerprint", "biometric", "template"

### 6. Geographic Coordinates

#### `GpsCoordinates`
**Priority**: Medium  
**Detection Method**: Pattern-based  
**Rationale**: Location privacy  
**Pattern Examples**:
- Decimal degrees: 40.7128, -74.0060
- DMS format: 40°42'46.0"N 74°00'21.6"W
- UTM coordinates

### 7. Vehicle Identifiers

#### `VehicleVin`
**Priority**: Medium  
**Detection Method**: Pattern-based + Checksum  
**Rationale**: Vehicle identification privacy  
**Pattern Examples**:
- 17-character VIN with check digit validation
- Exclude common invalid characters (I, O, Q)

#### `LicensePlate`
**Priority**: Medium  
**Detection Method**: Pattern-based + Context  
**Rationale**: Vehicle privacy  
**Pattern Examples**:
- State/country-specific formats
- Context: "plate", "license", vehicle descriptions

### 8. Education Identifiers

#### `StudentId`
**Priority**: Medium  
**Detection Method**: Pattern-based + Context  
**Rationale**: Educational record privacy (FERPA)  
**Pattern Examples**:
- Institution-specific formats
- Student numbers
- Context: "student ID", "enrollment number"

### 9. Employment Identifiers

#### `EmployeeId`
**Priority**: Medium  
**Detection Method**: Pattern-based + Context  
**Rationale**: HR privacy  
**Pattern Examples**:
- Company-specific employee numbers
- Badge numbers
- Context: "employee ID", "badge", "staff number"

### 10. Enhanced Patterns for Existing Types

#### Enhanced `PhoneNumber`
**Improvements**:
- International formats (+country codes)
- Extension handling
- VoIP numbers
- SMS short codes

#### Enhanced `Email`
**Improvements**:
- Internationalized domain names
- Plus addressing (email+tag@domain.com)
- Subdomain handling

#### Enhanced `IpAddress`
**Current**: Advanced IPv4/IPv6 detection with CIDR support
**Already Implemented**:
- ✅ CIDR notation support (IPv4/24, IPv6/64, etc.)
- ✅ IPv6 link-local addresses (fe80::/10)
- ✅ IPv6 compressed notation (::1, 2001:db8::1)
- ✅ IPv4-mapped IPv6 addresses (::ffff:192.0.2.1)
- ✅ Comprehensive validation using Rust std library
- ✅ Edge case handling for malformed IPs

**Potential Future Improvements**:
- Multicast address classification
- Reserved IP range categorization
- Geographic IP context detection

## Implementation Priority Ranking

### Tier 1 (High Priority - Immediate Implementation)
1. **BankAccountNumber** - Critical financial PII
2. **MedicalRecordNumber** - HIPAA compliance
3. **PassportNumber** - Travel document protection
4. **DriversLicense** - Common identification
5. **ApiKey** - Security credential protection
6. **TaxId** - Government identification
7. **BiometricTemplate** - Highly sensitive data

### Tier 2 (Medium Priority - Next Phase)
1. **InsuranceId** - Healthcare privacy
2. **MacAddress** - Device identification
3. **Uuid** - System identifiers
4. **CryptocurrencyAddress** - Financial privacy
5. **VehicleVin** - Vehicle identification
6. **GpsCoordinates** - Location privacy

### Tier 3 (Lower Priority - Future Enhancement)
1. **CreditScore** - Financial assessment data
2. **DrugId** - Prescription privacy
3. **VoterRegistration** - Electoral privacy
4. **LicensePlate** - Vehicle privacy
5. **StudentId** - Educational privacy
6. **EmployeeId** - HR privacy

## Technical Implementation Considerations

### Pattern-based Detection Requirements
- Regex pattern definitions
- Validation algorithms (checksums, format validation)
- False positive reduction
- Performance optimization

### ML-based Detection Requirements
- Training data availability
- GLiNER model compatibility
- Custom entity labeling
- Context-aware detection

### Anonymization Strategy Support
- Format-preserving options
- Pseudonymization pools
- Validation of anonymized outputs
- Reversibility requirements

### Compliance Considerations
- GDPR compliance for EU data
- HIPAA compliance for medical data
- SOX compliance for financial data
- Industry-specific regulations

## Next Steps

1. **Prioritize implementation** based on user needs and compliance requirements
2. **Develop pattern libraries** for Tier 1 entities
3. **Create test datasets** for validation
4. **Implement validation algorithms** for complex formats
5. **Add GLiNER support** where ML detection is beneficial
6. **Develop anonymization strategies** specific to each entity type
7. **Create comprehensive documentation** and examples

## Usage Examples

### Financial Data Protection
```rust
// Detect and anonymize financial identifiers
let entities = vec![
    EntityType::BankAccountNumber,
    EntityType::CreditCard,
    EntityType::TaxId,
    EntityType::CreditScore,
];

let result = anonymizer.anonymize_text(
    "Account: 123456789, Tax ID: 12-3456789, Credit Score: 780",
    &entities
)?;
// → "Account: [ACCOUNT], Tax ID: [TAX_ID], Credit Score: [CREDIT_SCORE]"
```

### Medical Record Protection
```rust
// Healthcare data anonymization
let entities = vec![
    EntityType::MedicalRecordNumber,
    EntityType::InsuranceId,
    EntityType::Person,
    EntityType::DrugId,
];

let result = anonymizer.anonymize_text(
    "Patient John Doe, MRN: 123456, Insurance: ABC123456, Prescribed: NDC 12345-678-90",
    &entities
)?;
// → "Patient [PERSON], MRN: [MRN], Insurance: [INSURANCE], Prescribed: [DRUG_ID]"
```

This expansion would significantly enhance the SDK's capability to detect and protect a wide range of sensitive data types across multiple domains and regulatory environments.