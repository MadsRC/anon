use crate::{AnonError, DataType, Dataset, Result};
use std::collections::HashSet;

pub fn validate_dataset(dataset: &Dataset) -> Result<()> {
    if dataset.data.is_empty() {
        return Err(AnonError::InsufficientData);
    }

    let mut row_counts = HashSet::new();
    for (column_name, values) in &dataset.data {
        row_counts.insert(values.len());

        if values.is_empty() {
            return Err(AnonError::InvalidInput(format!(
                "Column '{}' is empty",
                column_name
            )));
        }
    }

    if row_counts.len() > 1 {
        return Err(AnonError::InvalidInput(
            "All columns must have the same number of rows".to_string(),
        ));
    }

    Ok(())
}

pub fn calculate_diversity(values: &[String]) -> usize {
    let unique_values: HashSet<_> = values.iter().collect();
    unique_values.len()
}

pub fn is_numeric_string(s: &str) -> bool {
    s.parse::<f64>().is_ok()
}

pub fn infer_column_type(values: &[String]) -> DataType {
    if values.is_empty() {
        return DataType::Text;
    }

    let sample_size = std::cmp::min(100, values.len());
    let numeric_count = values
        .iter()
        .take(sample_size)
        .filter(|v| is_numeric_string(v))
        .count();

    if numeric_count as f64 / sample_size as f64 > 0.8 {
        DataType::Numeric
    } else {
        let unique_values: HashSet<_> = values.iter().take(sample_size).collect();
        if unique_values.len() < sample_size / 2 {
            DataType::Categorical
        } else {
            DataType::Text
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_diversity() {
        let values = vec![
            "A".to_string(),
            "B".to_string(),
            "A".to_string(),
            "C".to_string(),
        ];
        assert_eq!(calculate_diversity(&values), 3);
    }

    #[test]
    fn test_is_numeric_string() {
        assert!(is_numeric_string("123"));
        assert!(is_numeric_string("123.45"));
        assert!(is_numeric_string("-123.45"));
        assert!(!is_numeric_string("abc"));
        assert!(!is_numeric_string("12a"));
    }

    #[test]
    fn test_infer_column_type() {
        let numeric_values = vec!["1".to_string(), "2".to_string(), "3".to_string()];
        assert_eq!(infer_column_type(&numeric_values), DataType::Numeric);

        let categorical_values = vec![
            "A".to_string(),
            "A".to_string(),
            "B".to_string(),
            "B".to_string(),
            "A".to_string(),
            "B".to_string(),
        ];
        assert_eq!(
            infer_column_type(&categorical_values),
            DataType::Categorical
        );

        let text_values = vec![
            "Hello".to_string(),
            "World".to_string(),
            "Foo".to_string(),
            "Bar".to_string(),
        ];
        assert_eq!(infer_column_type(&text_values), DataType::Text);
    }
}
