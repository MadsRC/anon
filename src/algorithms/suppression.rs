use super::AnonymizationAlgorithm;
use crate::{AnonError, Dataset, Result};

#[derive(Debug, Clone)]
pub struct Suppression {
    columns: Vec<String>,
    suppression_char: String,
}

impl Suppression {
    pub fn new(columns: Vec<String>, suppression_char: Option<String>) -> Self {
        Self {
            columns,
            suppression_char: suppression_char.unwrap_or_else(|| "*".to_string()),
        }
    }
}

impl AnonymizationAlgorithm for Suppression {
    fn anonymize(&self, dataset: &Dataset) -> Result<Dataset> {
        self.validate_parameters()?;

        for column in &self.columns {
            if dataset.get_column(column).is_none() {
                return Err(AnonError::ColumnNotFound(column.clone()));
            }
        }

        let mut anonymized = dataset.clone();

        // Apply suppression to specified columns using suppression_char
        for column in &self.columns {
            if let Some(col_data) = anonymized.get_column_mut(column) {
                for i in 0..col_data.len() {
                    col_data[i] = self.suppression_char.repeat(col_data[i].len());
                }
            }
        }

        Ok(anonymized)
    }

    fn validate_parameters(&self) -> Result<()> {
        if self.columns.is_empty() {
            return Err(AnonError::InvalidInput(
                "At least one column must be specified for suppression".to_string(),
            ));
        }

        Ok(())
    }
}
