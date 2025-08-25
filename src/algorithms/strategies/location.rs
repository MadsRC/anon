use crate::algorithms::entity_anonymization::PseudonymPools;
use crate::{AnonError, Result};
use rand::Rng;
use rand::rngs::StdRng;

pub struct LocationStrategy;

impl LocationStrategy {
    pub fn redact(text: &str) -> String {
        // For locations, just mask
        "*".repeat(text.len())
    }

    pub fn suppress() -> String {
        "[LOCATION]".to_string()
    }

    pub fn generalize() -> String {
        "LOCATION".to_string()
    }

    pub fn pseudonymize(_text: &str, rng: &mut StdRng, pools: &PseudonymPools) -> Result<String> {
        let locations = pools
            .locations
            .as_ref()
            .ok_or_else(|| AnonError::InvalidInput("No locations pool provided".to_string()))?;

        if locations.is_empty() {
            return Err(AnonError::InvalidInput(
                "Empty locations pool provided".to_string(),
            ));
        }

        Ok(locations[rng.gen_range(0..locations.len())].clone())
    }
}
