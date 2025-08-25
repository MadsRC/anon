use crate::algorithms::entity_anonymization::PseudonymPools;
use crate::{AnonError, Result};
use rand::Rng;
use rand::rngs::StdRng;

pub struct EmailStrategy;

impl EmailStrategy {
    pub fn redact(text: &str) -> String {
        if let Some(at_pos) = text.find('@') {
            let (local_part, domain_part) = text.split_at(at_pos);
            format!("{}@{}", "*".repeat(local_part.len()), &domain_part[1..])
        } else {
            "*".repeat(text.len())
        }
    }

    pub fn suppress() -> String {
        "[EMAIL]".to_string()
    }

    pub fn generalize() -> String {
        "EMAIL".to_string()
    }

    pub fn pseudonymize(_text: &str, rng: &mut StdRng, pools: &PseudonymPools) -> Result<String> {
        let domains = pools
            .email_domains
            .as_ref()
            .ok_or_else(|| AnonError::InvalidInput("No email domains pool".to_string()))?;

        if domains.is_empty() {
            return Err(AnonError::InvalidInput(
                "Empty email domains pool provided".to_string(),
            ));
        }

        Ok(format!(
            "user{}@{}",
            rng.gen_range(1000..9999),
            domains[rng.gen_range(0..domains.len())]
        ))
    }
}
