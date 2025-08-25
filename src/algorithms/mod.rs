pub mod entity_anonymization;
pub mod entity_strategies;
pub mod generalization;
pub mod strategies;
pub mod suppression;
pub mod traits;

use crate::{Dataset, Result};

pub trait AnonymizationAlgorithm {
    fn anonymize(&self, dataset: &Dataset) -> Result<Dataset>;
    fn validate_parameters(&self) -> Result<()>;
}
