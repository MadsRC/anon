use crate::Result;
use crate::algorithms::entity_anonymization::PseudonymPools;
use crate::algorithms::strategies::*;
use crate::algorithms::traits::EntityAnonymizationStrategy;
use crate::detection::EntityType;
use rand::rngs::StdRng;

impl EntityAnonymizationStrategy for EntityType {
    fn redact(&self, text: &str) -> String {
        match self {
            EntityType::Email => EmailStrategy::redact(text),
            EntityType::PhoneNumber => PhoneStrategy::redact(text),
            EntityType::Person => PersonStrategy::redact(text),
            EntityType::Organization => OrganizationStrategy::redact(text),
            EntityType::Location => LocationStrategy::redact(text),
            EntityType::SocialSecurityNumber => SsnStrategy::redact(text),
            EntityType::CreditCard => CreditCardStrategy::redact(text),
            EntityType::IpAddress => IpAddressStrategy::redact(text),
            EntityType::Custom(_) => "*".repeat(text.len()),
        }
    }

    fn suppress(&self) -> String {
        match self {
            EntityType::Email => EmailStrategy::suppress(),
            EntityType::PhoneNumber => PhoneStrategy::suppress(),
            EntityType::Person => PersonStrategy::suppress(),
            EntityType::Organization => OrganizationStrategy::suppress(),
            EntityType::Location => LocationStrategy::suppress(),
            EntityType::SocialSecurityNumber => SsnStrategy::suppress(),
            EntityType::CreditCard => CreditCardStrategy::suppress(),
            EntityType::IpAddress => IpAddressStrategy::suppress(),
            EntityType::Custom(name) => format!("[{}]", name.to_uppercase()),
        }
    }

    fn generalize(&self) -> String {
        match self {
            EntityType::Email => EmailStrategy::generalize(),
            EntityType::PhoneNumber => PhoneStrategy::generalize(),
            EntityType::Person => PersonStrategy::generalize(),
            EntityType::Organization => OrganizationStrategy::generalize(),
            EntityType::Location => LocationStrategy::generalize(),
            EntityType::SocialSecurityNumber => SsnStrategy::generalize(),
            EntityType::CreditCard => CreditCardStrategy::generalize(),
            EntityType::IpAddress => IpAddressStrategy::generalize(),
            EntityType::Custom(name) => name.to_uppercase(),
        }
    }

    fn pseudonymize(&self, text: &str, rng: &mut StdRng, pools: &PseudonymPools) -> Result<String> {
        match self {
            EntityType::Email => EmailStrategy::pseudonymize(text, rng, pools),
            EntityType::PhoneNumber => PhoneStrategy::pseudonymize(text, rng, pools),
            EntityType::Person => PersonStrategy::pseudonymize(text, rng, pools),
            EntityType::Organization => OrganizationStrategy::pseudonymize(text, rng, pools),
            EntityType::Location => LocationStrategy::pseudonymize(text, rng, pools),
            EntityType::SocialSecurityNumber => SsnStrategy::pseudonymize(text, rng, pools),
            EntityType::CreditCard => CreditCardStrategy::pseudonymize(text, rng, pools),
            EntityType::IpAddress => IpAddressStrategy::pseudonymize(text, rng, pools),
            EntityType::Custom(name) => Ok(format!("[PSEUDO_{}]", name.to_uppercase())),
        }
    }
}
