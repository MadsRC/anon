use crate::algorithms::entity_anonymization::PseudonymPools;
use crate::{AnonError, Result};
use rand::Rng;
use rand::rngs::StdRng;

pub struct OrganizationStrategy;

impl OrganizationStrategy {
    pub fn redact(text: &str) -> String {
        // For organizations, keep some structure
        "*".repeat(text.len().min(10)) + " Corp"
    }

    pub fn suppress() -> String {
        "[ORGANIZATION]".to_string()
    }

    pub fn generalize() -> String {
        "ORGANIZATION".to_string()
    }

    pub fn pseudonymize(_text: &str, rng: &mut StdRng, pools: &PseudonymPools) -> Result<String> {
        let organizations = pools
            .organizations
            .as_ref()
            .ok_or_else(|| AnonError::InvalidInput("No organizations pool provided".to_string()))?;

        if organizations.is_empty() {
            return Err(AnonError::InvalidInput(
                "Empty organizations pool provided".to_string(),
            ));
        }

        Ok(organizations[rng.gen_range(0..organizations.len())].clone())
    }
}
