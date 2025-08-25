pub mod hybrid;
pub mod ner;
pub mod patterns;

use crate::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityType {
    Person,
    Location,
    Organization,
    Email,
    PhoneNumber,
    SocialSecurityNumber,
    CreditCard,
    IpAddress,
    Custom(String),
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityType::Person => write!(f, "person"),
            EntityType::Location => write!(f, "location"),
            EntityType::Organization => write!(f, "organization"),
            EntityType::Email => write!(f, "email"),
            EntityType::PhoneNumber => write!(f, "phone_number"),
            EntityType::SocialSecurityNumber => write!(f, "ssn"),
            EntityType::CreditCard => write!(f, "credit_card"),
            EntityType::IpAddress => write!(f, "ip_address"),
            EntityType::Custom(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetectedEntity {
    pub entity_type: EntityType,
    pub text: String,
    pub start: usize,
    pub end: usize,
    pub confidence: f32,
}

impl DetectedEntity {
    pub fn new(
        entity_type: EntityType,
        text: String,
        start: usize,
        end: usize,
        confidence: f32,
    ) -> Self {
        Self {
            entity_type,
            text,
            start,
            end,
            confidence,
        }
    }

    pub fn length(&self) -> usize {
        self.end - self.start
    }
}

pub trait EntityDetector {
    fn detect(&mut self, text: &str) -> Result<Vec<DetectedEntity>>;
    fn supported_entities(&self) -> Vec<EntityType>;
    fn as_any(&self) -> &dyn std::any::Any;
}
