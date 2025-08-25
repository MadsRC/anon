use crate::Result;
use crate::algorithms::entity_anonymization::PseudonymPools;
use rand::Rng;
use rand::rngs::StdRng;

pub struct CreditCardStrategy;

impl CreditCardStrategy {
    pub fn redact(text: &str) -> String {
        let digits: String = text.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() >= 4 {
            format!("****-****-****-{}", &digits[digits.len() - 4..])
        } else {
            "*".repeat(text.len())
        }
    }

    pub fn suppress() -> String {
        "[CREDIT_CARD]".to_string()
    }

    pub fn generalize() -> String {
        "CREDIT_CARD".to_string()
    }

    pub fn pseudonymize(_text: &str, rng: &mut StdRng, _pools: &PseudonymPools) -> Result<String> {
        // Credit cards don't need pools - use algorithmic generation
        Ok(format!("****-****-****-{:04}", rng.gen_range(1000..9999)))
    }
}
