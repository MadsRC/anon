pub mod differential_privacy;
pub mod entity_anonymization;
pub mod generalization;
pub mod k_anonymity;
pub mod suppression;

use crate::{Dataset, Result};

pub trait AnonymizationAlgorithm {
    fn anonymize(&self, dataset: &Dataset) -> Result<Dataset>;
    fn validate_parameters(&self) -> Result<()>;
}
