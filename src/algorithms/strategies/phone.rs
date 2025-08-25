use crate::Result;
use crate::algorithms::entity_anonymization::PseudonymPools;
use rand::Rng;
use rand::rngs::StdRng;

pub struct PhoneStrategy;

impl PhoneStrategy {
    pub fn redact(text: &str) -> String {
        text.chars()
            .map(|c| if c.is_ascii_digit() { '*' } else { c })
            .collect()
    }

    pub fn suppress() -> String {
        "[PHONE]".to_string()
    }

    pub fn generalize() -> String {
        "PHONE".to_string()
    }

    pub fn pseudonymize(_text: &str, rng: &mut StdRng, _pools: &PseudonymPools) -> Result<String> {
        // Phone numbers don't need pools - use algorithmic generation
        Ok(format!(
            "555-{:03}-{:04}",
            rng.gen_range(100..999),
            rng.gen_range(1000..9999)
        ))
    }
}
