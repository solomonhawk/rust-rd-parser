use crate::ast::{Expression, RuleContent, Table};
use crate::parse;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use thiserror::Error;

#[cfg(feature = "wasm")]
type HashMapType<K, V> = std::collections::HashMap<K, V, ahash::RandomState>;
#[cfg(not(feature = "wasm"))]
type HashMapType<K, V> = std::collections::HashMap<K, V>;

/// Optimized table for fast generation with pre-computed weights
#[derive(Debug, Clone)]
struct OptimizedTable {
    pub metadata: crate::ast::TableMetadata,
    pub rules: Vec<crate::ast::Node<crate::ast::Rule>>,
    /// Pre-computed cumulative weights for O(log n) weighted selection via binary search
    pub cumulative_weights: Vec<f64>,
    /// Total weight of all rules (cached for performance)
    pub total_weight: f64,
}

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

    #[error(
        "Invalid table reference: Table '{table_id}' referenced in table '{referencing_table}' does not exist"
    )]
    InvalidTableReference {
        table_id: String,
        referencing_table: String,
    },

    #[error(
        "Missing dependency: External reference '@{publisher}/{collection}#{table_id}' in table '{referencing_table}' cannot be resolved. The collection '@{publisher}/{collection}' was not provided in the input sources."
    )]
    MissingDependency {
        publisher: String,
        collection: String,
        table_id: String,
        referencing_table: String,
    },

    #[error(
        "External table not found: External reference '@{publisher}/{collection}#{table_id}' in table '{referencing_table}' refers to a table that does not exist in the provided collection."
    )]
    ExternalTableNotFound {
        publisher: String,
        collection: String,
        table_id: String,
        referencing_table: String,
    },
}

/// Result type for collection operations
pub type CollectionResult<T> = Result<T, CollectionError>;

/// Result type specifically for generation operations
pub type CollectionGenResult = CollectionResult<String>;

impl OptimizedTable {
    /// Create an optimized table from a parsed table with pre-computed weights
    fn from_table(table: Table) -> CollectionResult<Self> {
        if table.rules.is_empty() {
            return Err(CollectionError::EmptyTable(table.metadata.id.clone()));
        }

        let mut cumulative_weights = Vec::with_capacity(table.rules.len());
        let mut cumulative = 0.0;

        // Pre-compute cumulative weights for O(log n) binary search during generation
        for rule in &table.rules {
            cumulative += rule.value.weight;
            cumulative_weights.push(cumulative);
        }

        let total_weight = cumulative;

        Ok(Self {
            metadata: table.metadata,
            rules: table.rules,
            cumulative_weights,
            total_weight,
        })
    }

    /// Fast weighted rule selection using binary search on pre-computed cumulative weights
    /// This is O(log n) instead of O(n) linear search
    fn select_rule_index(&self, random_value: f64) -> usize {
        match self.cumulative_weights.binary_search_by(|&weight| {
            if weight < random_value {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        }) {
            Ok(index) => index,
            Err(index) => index.min(self.rules.len() - 1),
        }
    }
}

/// A collection of tables that can generate random content
#[derive(Debug)]
pub struct Collection {
    tables: HashMapType<String, OptimizedTable>,
    rng: SmallRng,
    table_order: Vec<String>, // Preserve the order tables appear in source
}

impl Collection {
    /// Create a new collection from TBL source code
    pub fn new(source: &str) -> CollectionResult<Self> {
        let program = parse(source).map_err(|e| CollectionError::ParseError(format!("{}", e)))?;

        #[cfg(feature = "wasm")]
        let mut tables = HashMapType::with_hasher(ahash::RandomState::new());
        #[cfg(not(feature = "wasm"))]
        let mut tables = HashMapType::default();
        let mut table_order = Vec::new();

        // First pass: collect all tables and preserve order, optimizing during parse-time
        for table_node in program.tables {
            let table = table_node.value;
            let table_id = table.metadata.id.clone();

            // Convert to optimized table with pre-computed weights (parse-time optimization)
            let optimized_table = OptimizedTable::from_table(table)?;

            table_order.push(table_id.clone());
            tables.insert(table_id, optimized_table);
        }

        // Second pass: validate all table references
        Self::validate_table_references(&tables)?;

        Ok(Self {
            tables,
            rng: SmallRng::seed_from_u64(rand::random::<u64>()), // Use random seed
            table_order,
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

    /// Generate a single result from a table (now optimized with pre-computed weights)
    fn generate_single(&mut self, table_id: &str) -> CollectionResult<String> {
        // Get the rule using optimized selection
        let rule_content = {
            let table = self
                .tables
                .get(table_id)
                .ok_or_else(|| CollectionError::TableNotFound(table_id.to_string()))?;

            // Use pre-computed total weight (O(1) instead of O(n))
            let random_value: f64 = self.rng.gen_range(0.0..table.total_weight);

            // Use binary search on pre-computed cumulative weights (O(log n) instead of O(n))
            let rule_index = table.select_rule_index(random_value);
            let selected_rule = &table.rules[rule_index];

            // Clone the content so we don't hold a reference to self
            selected_rule.value.content.clone()
        };

        // Process the rule content
        let mut result = String::new();

        for content in &rule_content {
            match content {
                RuleContent::Text(text) => {
                    result.push_str(text);
                }
                RuleContent::Expression(Expression::TableReference {
                    table_id: ref_id,
                    modifiers,
                }) => {
                    // Recursively generate from the referenced table
                    let mut generated = self.generate_single(ref_id)?;

                    // Apply modifiers
                    for modifier in modifiers {
                        generated = self.apply_modifier(&generated, modifier);
                    }

                    result.push_str(&generated);
                }
                RuleContent::Expression(Expression::ExternalTableReference {
                    publisher,
                    collection,
                    table_id,
                    modifiers: _,
                }) => {
                    // For now, external references always error since we don't have dependency resolution
                    // In the future, this will be handled by the dependency resolution system
                    return Err(CollectionError::MissingDependency {
                        publisher: publisher.clone(),
                        collection: collection.clone(),
                        table_id: table_id.clone(),
                        referencing_table: table_id.clone(), // TODO: we need to pass the current table being generated
                    });
                }
                RuleContent::Expression(Expression::DiceRoll { count, sides }) => {
                    // Roll dice and add the result
                    let dice_count = count.unwrap_or(1);
                    let mut total = 0;
                    for _ in 0..dice_count {
                        total += self.rng.gen_range(1..=*sides);
                    }
                    result.push_str(&total.to_string());
                }
            }
        }

        Ok(result.trim().to_string())
    }

    /// Apply a modifier to generated text
    fn apply_modifier(&self, text: &str, modifier: &str) -> String {
        match modifier {
            "capitalize" => {
                let mut chars: Vec<char> = text.chars().collect();
                if let Some(first_char) = chars.get_mut(0) {
                    *first_char = first_char.to_uppercase().next().unwrap_or(*first_char);
                }
                chars.into_iter().collect()
            }
            "uppercase" => text.to_uppercase(),
            "lowercase" => text.to_lowercase(),
            "indefinite" => {
                let first_char = text
                    .chars()
                    .next()
                    .unwrap_or(' ')
                    .to_lowercase()
                    .next()
                    .unwrap_or(' ');
                let article = if "aeiou".contains(first_char) {
                    "an"
                } else {
                    "a"
                };
                format!("{} {}", article, text)
            }
            "definite" => format!("the {}", text),
            _ => text.to_string(), // Unknown modifier, return unchanged
        }
    }

    /// Validate that all table references point to existing tables
    fn validate_table_references(
        tables: &HashMapType<String, OptimizedTable>,
    ) -> CollectionResult<()> {
        for (table_id, table) in tables {
            for rule in &table.rules {
                for content in &rule.value.content {
                    match content {
                        RuleContent::Expression(Expression::TableReference {
                            table_id: ref_id,
                            modifiers: _,
                        }) => {
                            if !tables.contains_key(ref_id) {
                                return Err(CollectionError::InvalidTableReference {
                                    table_id: ref_id.clone(),
                                    referencing_table: table_id.clone(),
                                });
                            }
                        }
                        RuleContent::Expression(Expression::ExternalTableReference {
                            publisher,
                            collection,
                            table_id: ext_table_id,
                            modifiers: _,
                        }) => {
                            // External references always error in basic collections since dependencies aren't resolved
                            return Err(CollectionError::MissingDependency {
                                publisher: publisher.clone(),
                                collection: collection.clone(),
                                table_id: ext_table_id.clone(),
                                referencing_table: table_id.clone(),
                            });
                        }
                        _ => {} // Other content types (text, dice rolls) don't need validation
                    }
                }
            }
        }
        Ok(())
    }

    /// Check if a table exists in the collection
    pub fn has_table(&self, table_id: &str) -> bool {
        self.tables.contains_key(table_id)
    }

    /// Get a list of all table IDs in the collection
    pub fn get_table_ids(&self) -> Vec<String> {
        // Return table IDs in the order they appear in the source
        self.table_order.clone()
    }

    /// Get a list of exported table IDs in the collection
    pub fn get_exported_table_ids(&self) -> Vec<String> {
        // Return exported table IDs in the order they appear in the source
        self.table_order
            .iter()
            .filter(|table_id| {
                self.tables
                    .get(*table_id)
                    .map(|table| table.metadata.export)
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
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

    #[test]
    fn test_valid_table_references() {
        let source = r#"#color
1.0: red
2.0: blue

#shape
1.0: circle
2.0: square

#item
1.0: {#color} {#shape}"#;

        let collection = Collection::new(source);
        assert!(
            collection.is_ok(),
            "Valid table references should be accepted"
        );
    }

    #[test]
    fn test_invalid_table_reference() {
        let source = r#"#color
1.0: red
2.0: blue

#item
1.0: {#nonexistent} shape"#;

        let collection = Collection::new(source);
        assert!(
            collection.is_err(),
            "Invalid table reference should cause error"
        );

        if let Err(CollectionError::InvalidTableReference {
            table_id,
            referencing_table,
        }) = collection
        {
            assert_eq!(table_id, "nonexistent");
            assert_eq!(referencing_table, "item");
        } else {
            panic!("Expected InvalidTableReference error");
        }
    }

    #[test]
    fn test_multiple_invalid_references() {
        let source = r#"#color
1.0: red

#item
1.0: {#missing1} {#missing2}"#;

        let collection = Collection::new(source);
        assert!(
            collection.is_err(),
            "Invalid table references should cause error"
        );

        // Should fail on the first invalid reference
        if let Err(CollectionError::InvalidTableReference {
            table_id,
            referencing_table,
        }) = collection
        {
            assert_eq!(table_id, "missing1");
            assert_eq!(referencing_table, "item");
        } else {
            panic!("Expected InvalidTableReference error");
        }
    }

    #[test]
    fn test_self_reference() {
        let source = r#"#color
1.0: {#color} variant"#;

        let collection = Collection::new(source);
        assert!(collection.is_ok(), "Self-references should be valid");
    }

    #[test]
    fn test_table_ids_order() {
        let source = r#"#zebra
1.0: striped

#alpha
1.0: first

#beta[export]
1.0: second"#;

        let collection = Collection::new(source).unwrap();
        let table_ids = collection.get_table_ids();

        // Should return tables in source order, not alphabetical
        assert_eq!(table_ids, vec!["zebra", "alpha", "beta"]);

        let exported_ids = collection.get_exported_table_ids();
        assert_eq!(exported_ids, vec!["beta"]);
    }
}
