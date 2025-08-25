use crate::Result;
use crate::algorithms::entity_anonymization::PseudonymPools;
use rand::Rng;
use rand::rngs::StdRng;

pub struct SsnStrategy;

impl SsnStrategy {
    pub fn redact(text: &str) -> String {
        text.chars()
            .map(|c| if c.is_ascii_digit() { '*' } else { c })
            .collect()
    }

    pub fn suppress() -> String {
        "[SSN]".to_string()
    }

    pub fn generalize() -> String {
        "SSN".to_string()
    }

    pub fn pseudonymize(_text: &str, rng: &mut StdRng, _pools: &PseudonymPools) -> Result<String> {
        // SSN doesn't need pools - use algorithmic generation
        Ok(format!(
            "{:03}-{:02}-{:04}",
            rng.gen_range(100..999),
            rng.gen_range(10..99),
            rng.gen_range(1000..9999)
        ))
    }
}
