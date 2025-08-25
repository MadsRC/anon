use super::AnonymizationAlgorithm;
use crate::{AnonError, Dataset, Result};

#[derive(Debug, Clone)]
pub struct KAnonymity {
    k: usize,
    quasi_identifiers: Vec<String>,
}

impl KAnonymity {
    pub fn new(k: usize, quasi_identifiers: Vec<String>) -> Result<Self> {
        if k < 2 {
            return Err(AnonError::InvalidPrivacyParameter(
                "k must be at least 2".to_string(),
            ));
        }

        Ok(Self {
            k,
            quasi_identifiers,
        })
    }

    pub fn k(&self) -> usize {
        self.k
    }

    pub fn quasi_identifiers(&self) -> &[String] {
        &self.quasi_identifiers
    }
}

impl AnonymizationAlgorithm for KAnonymity {
    fn anonymize(&self, dataset: &Dataset) -> Result<Dataset> {
        self.validate_parameters()?;

        for qi in &self.quasi_identifiers {
            if dataset.get_column(qi).is_none() {
                return Err(AnonError::ColumnNotFound(qi.clone()));
            }
        }

        Ok(dataset.clone())
    }

    fn validate_parameters(&self) -> Result<()> {
        if self.k < 2 {
            return Err(AnonError::InvalidPrivacyParameter(
                "k must be at least 2".to_string(),
            ));
        }

        if self.quasi_identifiers.is_empty() {
            return Err(AnonError::InvalidInput(
                "At least one quasi-identifier must be specified".to_string(),
            ));
        }

        Ok(())
    }
}
