pub mod algorithms;
pub mod cli;
pub mod detection;
pub mod error;
pub mod evaluation;
pub mod pool_manager;
pub mod utils;

pub use error::{AnonError, Result};

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Numeric,
    Categorical,
    Text,
    DateTime,
}

#[derive(Debug, Clone)]
pub struct Dataset {
    data: HashMap<String, Vec<String>>,
    column_types: HashMap<String, DataType>,
}

impl Default for Dataset {
    fn default() -> Self {
        Self::new()
    }
}

impl Dataset {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            column_types: HashMap::new(),
        }
    }

    pub fn add_column(&mut self, name: String, values: Vec<String>, data_type: DataType) {
        self.data.insert(name.clone(), values);
        self.column_types.insert(name, data_type);
    }

    pub fn get_column(&self, name: &str) -> Option<&Vec<String>> {
        self.data.get(name)
    }

    pub fn get_column_mut(&mut self, name: &str) -> Option<&mut Vec<String>> {
        self.data.get_mut(name)
    }

    pub fn get_column_type(&self, name: &str) -> Option<&DataType> {
        self.column_types.get(name)
    }
}
