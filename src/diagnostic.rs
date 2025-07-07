/// Diagnostic system for collecting and reporting parse errors
///
/// This module provides a clean separation between error data collection
/// and error formatting/rendering.
/// Source location information
#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    pub position: usize,
    pub line: usize,
    pub column: usize,
    /// Optional end position for span-based diagnostics
    pub end_position: Option<usize>,
    /// Optional end column for span-based diagnostics  
    pub end_column: Option<usize>,
}

/// A diagnostic represents a structured error with source context
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub location: SourceLocation,
    pub message: String,
    pub suggestion: Option<String>,
    pub source_line: String,
}

/// Different categories of diagnostics
#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticKind {
    /// Lexical analysis errors
    LexError,
    /// Parsing errors
    ParseError,
    /// Semantic analysis errors (for future use)
    SemanticError,
}

/// Severity levels for diagnostics
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

impl Diagnostic {
    pub fn new(
        kind: DiagnosticKind,
        location: SourceLocation,
        message: String,
        source_line: String,
    ) -> Self {
        Self {
            kind,
            location,
            message,
            suggestion: None,
            source_line,
        }
    }

    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }

    pub fn severity(&self) -> Severity {
        match self.kind {
            DiagnosticKind::LexError
            | DiagnosticKind::ParseError
            | DiagnosticKind::SemanticError => Severity::Error,
        }
    }
}
