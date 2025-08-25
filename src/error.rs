use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnonError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Column not found: {0}")]
    ColumnNotFound(String),

    #[error("Unsupported data type for operation: {0}")]
    UnsupportedDataType(String),

    #[error("Privacy parameter out of range: {0}")]
    InvalidPrivacyParameter(String),

    #[error("Insufficient data for anonymization")]
    InsufficientData,

    #[error("Algorithm error: {0}")]
    AlgorithmError(String),
}

pub type Result<T> = std::result::Result<T, AnonError>;
