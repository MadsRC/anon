use super::AnonymizationAlgorithm;
use crate::{AnonError, Dataset, Result};
use rand::{Rng, thread_rng};

#[derive(Debug, Clone)]
pub struct DifferentialPrivacy {
    epsilon: f64,
    sensitivity: f64,
}

impl DifferentialPrivacy {
    pub fn new(epsilon: f64, sensitivity: f64) -> Result<Self> {
        if epsilon <= 0.0 {
            return Err(AnonError::InvalidPrivacyParameter(
                "Epsilon must be positive".to_string(),
            ));
        }

        if sensitivity <= 0.0 {
            return Err(AnonError::InvalidPrivacyParameter(
                "Sensitivity must be positive".to_string(),
            ));
        }

        Ok(Self {
            epsilon,
            sensitivity,
        })
    }

    pub fn epsilon(&self) -> f64 {
        self.epsilon
    }

    pub fn sensitivity(&self) -> f64 {
        self.sensitivity
    }

    pub fn add_laplace_noise(&self, value: f64) -> f64 {
        let mut rng = thread_rng();
        let scale = self.sensitivity / self.epsilon;

        let u: f64 = rng.gen_range(-1.0..1.0);
        let noise = if u >= 0.0 {
            -scale * (1.0 - u).ln()
        } else {
            scale * (1.0 + u).ln()
        };

        value + noise
    }
}

impl AnonymizationAlgorithm for DifferentialPrivacy {
    fn anonymize(&self, dataset: &Dataset) -> Result<Dataset> {
        self.validate_parameters()?;

        let anonymized = dataset.clone();

        Ok(anonymized)
    }

    fn validate_parameters(&self) -> Result<()> {
        if self.epsilon <= 0.0 {
            return Err(AnonError::InvalidPrivacyParameter(
                "Epsilon must be positive".to_string(),
            ));
        }

        if self.sensitivity <= 0.0 {
            return Err(AnonError::InvalidPrivacyParameter(
                "Sensitivity must be positive".to_string(),
            ));
        }

        Ok(())
    }
}
