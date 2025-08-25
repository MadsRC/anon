use crate::algorithms::entity_anonymization::PseudonymPools;
use crate::{AnonError, Result};
use rand::Rng;
use rand::rngs::StdRng;

pub struct PersonStrategy;

impl PersonStrategy {
    pub fn redact(text: &str) -> String {
        // For names, preserve structure (First Last -> ***** ****)
        if text.contains(' ') {
            text.split_whitespace()
                .map(|word| "*".repeat(word.len()))
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            "*".repeat(text.len())
        }
    }

    pub fn suppress() -> String {
        "[PERSON]".to_string()
    }

    pub fn generalize() -> String {
        "PERSON".to_string()
    }

    pub fn pseudonymize(_text: &str, rng: &mut StdRng, pools: &PseudonymPools) -> Result<String> {
        let first_names = pools
            .first_names
            .as_ref()
            .ok_or_else(|| AnonError::InvalidInput("No first names pool provided".to_string()))?;
        let last_names = pools
            .last_names
            .as_ref()
            .ok_or_else(|| AnonError::InvalidInput("No last names pool provided".to_string()))?;

        if first_names.is_empty() || last_names.is_empty() {
            return Err(AnonError::InvalidInput(
                "Empty name pools provided".to_string(),
            ));
        }

        Ok(format!(
            "{} {}",
            first_names[rng.gen_range(0..first_names.len())],
            last_names[rng.gen_range(0..last_names.len())]
        ))
    }
}
