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

/// A single rule in our language: weight: rule_text
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rule {
    pub weight: f64,
    pub text: String,
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
        write!(f, "{}: {}", self.weight, self.text)
    }
}
