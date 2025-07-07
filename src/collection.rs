use crate::ast::{Expression, RuleContent, Table};
use crate::parse;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during collection generation
#[derive(Error, Debug)]
pub enum CollectionError {
    #[error("Table '{0}' not found")]
    TableNotFound(String),

    #[error("Table '{0}' has no rules")]
    EmptyTable(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Generation error: {0}")]
    GenerationError(String),
}

/// Result type for collection operations
pub type CollectionResult<T> = Result<T, CollectionError>;

/// Result type specifically for generation operations
pub type CollectionGenResult = CollectionResult<String>;

/// A collection of tables that can generate random content
pub struct Collection {
    tables: HashMap<String, Table>,
    distributions: HashMap<String, WeightedIndex<f64>>,
    rng: ThreadRng,
}

impl Collection {
    /// Create a new collection from TBL source code
    pub fn new(source: &str) -> CollectionResult<Self> {
        let program = parse(source).map_err(|e| CollectionError::ParseError(format!("{}", e)))?;

        let mut tables = HashMap::new();
        let mut distributions = HashMap::new();

        for table_node in program.tables {
            let table = table_node.value;
            let table_id = table.metadata.id.clone();

            // Extract weights from rules
            let weights: Vec<f64> = table.rules.iter().map(|rule| rule.value.weight).collect();

            if weights.is_empty() {
                return Err(CollectionError::EmptyTable(table_id));
            }

            let distribution = WeightedIndex::new(&weights)
                .map_err(|e| CollectionError::GenerationError(format!("Invalid weights: {}", e)))?;
            distributions.insert(table_id.clone(), distribution);
            tables.insert(table_id, table);
        }

        Ok(Self {
            tables,
            distributions,
            rng: thread_rng(),
        })
    }

    /// Generate content from a table by ID
    pub fn generate(&mut self, table_id: &str, count: usize) -> CollectionGenResult {
        let mut results = Vec::new();

        for _ in 0..count {
            let result = self.generate_single(table_id)?;
            results.push(result);
        }

        Ok(results.join(", "))
    }

    /// Generate a single result from a table
    fn generate_single(&mut self, table_id: &str) -> CollectionResult<String> {
        // Get the rule first without holding references to self
        let rule_content = {
            let table = self
                .tables
                .get(table_id)
                .ok_or_else(|| CollectionError::TableNotFound(table_id.to_string()))?;

            let distribution = self
                .distributions
                .get(table_id)
                .ok_or_else(|| CollectionError::TableNotFound(table_id.to_string()))?;

            // Sample a rule using the weighted distribution
            let rule_index = distribution.sample(&mut self.rng);

            if rule_index >= table.rules.len() {
                return Err(CollectionError::GenerationError(
                    "Invalid rule index".to_string(),
                ));
            }

            let rule = &table.rules[rule_index].value;

            // Clone the content so we don't hold a reference to self
            rule.content.clone()
        };

        // Process the rule content
        let mut result = String::new();

        for content in &rule_content {
            match content {
                RuleContent::Text(text) => {
                    result.push_str(text);
                }
                RuleContent::Expression(Expression::TableReference { table_id: ref_id }) => {
                    // Recursively generate from the referenced table
                    let generated = self.generate_single(ref_id)?;
                    result.push_str(&generated);
                }
            }
        }

        Ok(result.trim().to_string())
    }

    /// Check if a table exists in the collection
    pub fn has_table(&self, table_id: &str) -> bool {
        self.tables.contains_key(table_id)
    }

    /// Get a list of all table IDs in the collection
    pub fn get_table_ids(&self) -> Vec<String> {
        self.tables.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_creation() {
        let source = r#"#color
1.0: red
2.0: blue
3.0: green"#;

        let collection = Collection::new(source);
        assert!(collection.is_ok());

        let collection = collection.unwrap();
        assert!(collection.tables.contains_key("color"));
    }

    #[test]
    fn test_simple_generation() {
        let source = r#"#color
1.0: red
2.0: blue
3.0: green"#;

        let mut collection = Collection::new(source).unwrap();
        let result = collection.generate("color", 1);
        assert!(result.is_ok());

        let generated = result.unwrap();
        assert!(generated == "red" || generated == "blue" || generated == "green");
    }

    #[test]
    fn test_table_reference() {
        let source = r#"#color
1.0: red
2.0: blue

#shape
1.0: circle
2.0: square

#item
1.0: {#color} {#shape}"#;

        let mut collection = Collection::new(source).unwrap();
        let result = collection.generate("item", 1);
        assert!(result.is_ok());

        let generated = result.unwrap();
        // Should contain a color and a shape
        assert!(generated.contains("red") || generated.contains("blue"));
        assert!(generated.contains("circle") || generated.contains("square"));
    }

    #[test]
    fn test_multiple_generation() {
        let source = r#"#color
1.0: red"#;

        let mut collection = Collection::new(source).unwrap();
        let result = collection.generate("color", 3);
        assert!(result.is_ok());

        let generated = result.unwrap();
        assert_eq!(generated, "red, red, red");
    }

    #[test]
    fn test_table_not_found() {
        let source = r#"#color
1.0: red"#;

        let mut collection = Collection::new(source).unwrap();
        let result = collection.generate("nonexistent", 1);
        assert!(result.is_err());

        if let Err(CollectionError::TableNotFound(id)) = result {
            assert_eq!(id, "nonexistent");
        } else {
            panic!("Expected TableNotFound error");
        }
    }
}
