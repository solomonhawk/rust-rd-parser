use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Represents a position in the source code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// A node in the AST with position information
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Node<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Node<T> {
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }
}

/// Expression that can appear within rule text
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Expression {
    /// Reference to another table by ID with optional modifiers
    TableReference {
        table_id: String,
        modifiers: Vec<String>,
    },
    /// Reference to a table in an external collection
    ExternalTableReference {
        publisher: String,      // @username
        collection: String,     // collection name
        table_id: String,       // table within that collection
        modifiers: Vec<String>, // same modifiers as internal refs
    },
    /// Dice roll expression like "d6", "2d10", "100d20"
    DiceRoll { count: Option<u32>, sides: u32 },
}

/// A piece of rule text content - either literal text or an expression
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum RuleContent {
    /// Literal text content
    Text(String),
    /// An expression to be evaluated
    Expression(Expression),
}

/// A single rule in our language: weight: rule_content_list
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rule {
    pub weight: f64,
    pub content: Vec<RuleContent>,
}

impl Rule {
    /// Create a new rule with text content (for backward compatibility)
    pub fn new_text(weight: f64, text: String) -> Self {
        Self {
            weight,
            content: vec![RuleContent::Text(text)],
        }
    }

    /// Create a new rule with mixed content
    pub fn new(weight: f64, content: Vec<RuleContent>) -> Self {
        Self { weight, content }
    }

    /// Get just the content text without weight and colon (for backward compatibility)
    pub fn content_text(&self) -> String {
        self.content
            .iter()
            .map(|c| match c {
                RuleContent::Text(text) => text.clone(),
                RuleContent::Expression(Expression::TableReference {
                    table_id,
                    modifiers,
                }) => {
                    if modifiers.is_empty() {
                        format!("{{#{}}}", table_id)
                    } else {
                        format!("{{#{}|{}}}", table_id, modifiers.join("|"))
                    }
                }
                RuleContent::Expression(Expression::ExternalTableReference {
                    publisher,
                    collection,
                    table_id,
                    modifiers,
                }) => {
                    if modifiers.is_empty() {
                        format!("{{@{}/{}#{}}}", publisher, collection, table_id)
                    } else {
                        format!(
                            "{{@{}/{}#{}|{}}}",
                            publisher,
                            collection,
                            table_id,
                            modifiers.join("|")
                        )
                    }
                }
                RuleContent::Expression(Expression::DiceRoll { count, sides }) => match count {
                    Some(c) => format!("{{{}d{}}}", c, sides),
                    None => format!("{{d{}}}", sides),
                },
            })
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string()
    }
}

/// Table metadata containing id and optional flags
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TableMetadata {
    pub id: String,
    pub export: bool,
}

impl TableMetadata {
    pub fn new(id: String) -> Self {
        Self { id, export: false }
    }

    pub fn with_export(mut self, export: bool) -> Self {
        self.export = export;
        self
    }
}

/// A table containing metadata and a list of rules
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Table {
    pub metadata: TableMetadata,
    pub rules: Vec<Node<Rule>>,
}

impl Table {
    pub fn new(metadata: TableMetadata, rules: Vec<Node<Rule>>) -> Self {
        Self { metadata, rules }
    }
}

/// The root of the AST - a TBL program containing multiple tables
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Program {
    pub tables: Vec<Node<Table>>,
}

impl Program {
    pub fn new(tables: Vec<Node<Table>>) -> Self {
        Self { tables }
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let content_str = self
            .content
            .iter()
            .map(|c| match c {
                RuleContent::Text(text) => text.clone(),
                RuleContent::Expression(Expression::TableReference {
                    table_id,
                    modifiers,
                }) => {
                    if modifiers.is_empty() {
                        format!("{{#{}}}", table_id)
                    } else {
                        format!("{{#{}|{}}}", table_id, modifiers.join("|"))
                    }
                }
                RuleContent::Expression(Expression::ExternalTableReference {
                    publisher,
                    collection,
                    table_id,
                    modifiers,
                }) => {
                    if modifiers.is_empty() {
                        format!("{{@{}/{}#{}}}", publisher, collection, table_id)
                    } else {
                        format!(
                            "{{@{}/{}#{}|{}}}",
                            publisher,
                            collection,
                            table_id,
                            modifiers.join("|")
                        )
                    }
                }
                RuleContent::Expression(Expression::DiceRoll { count, sides }) => match count {
                    Some(c) => format!("{{{}d{}}}", c, sides),
                    None => format!("{{d{}}}", sides),
                },
            })
            .collect::<Vec<_>>()
            .join("");
        write!(f, "{}: {}", self.weight, content_str)
    }
}
