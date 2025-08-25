use super::AnonymizationAlgorithm;
use crate::{AnonError, Dataset, Result};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Generalization {
    hierarchies: HashMap<String, Vec<Vec<String>>>,
}

impl Generalization {
    pub fn new() -> Self {
        Self {
            hierarchies: HashMap::new(),
        }
    }

    pub fn add_hierarchy(&mut self, column: String, hierarchy: Vec<Vec<String>>) -> Result<()> {
        if hierarchy.is_empty() {
            return Err(AnonError::InvalidInput(
                "Hierarchy cannot be empty".to_string(),
            ));
        }

        self.hierarchies.insert(column, hierarchy);
        Ok(())
    }

    pub fn get_hierarchy(&self, column: &str) -> Option<&Vec<Vec<String>>> {
        self.hierarchies.get(column)
    }
}

impl AnonymizationAlgorithm for Generalization {
    fn anonymize(&self, dataset: &Dataset) -> Result<Dataset> {
        self.validate_parameters()?;

        for column in self.hierarchies.keys() {
            if dataset.get_column(column).is_none() {
                return Err(AnonError::ColumnNotFound(column.clone()));
            }
        }

        let anonymized = dataset.clone();

        Ok(anonymized)
    }

    fn validate_parameters(&self) -> Result<()> {
        if self.hierarchies.is_empty() {
            return Err(AnonError::InvalidInput(
                "At least one generalization hierarchy must be defined".to_string(),
            ));
        }

        for (column, hierarchy) in &self.hierarchies {
            if hierarchy.is_empty() {
                return Err(AnonError::InvalidInput(format!(
                    "Hierarchy for column '{}' cannot be empty",
                    column
                )));
            }
        }

        Ok(())
    }
}
