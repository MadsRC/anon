// Individual strategy modules
pub mod credit_card;
pub mod email;
pub mod ip_address;
pub mod location;
pub mod organization;
pub mod person;
pub mod phone;
pub mod ssn;

// Re-export strategy structs
pub use credit_card::CreditCardStrategy;
pub use email::EmailStrategy;
pub use ip_address::IpAddressStrategy;
pub use location::LocationStrategy;
pub use organization::OrganizationStrategy;
pub use person::PersonStrategy;
pub use phone::PhoneStrategy;
pub use ssn::SsnStrategy;
