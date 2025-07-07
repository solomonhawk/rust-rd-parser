use crate::diagnostic::{Diagnostic, DiagnosticKind, SourceLocation};

/// Collects diagnostic information from source code
pub struct DiagnosticCollector {
    source: String,
}

impl DiagnosticCollector {
    pub fn new(source: String) -> Self {
        Self { source }
    }

    /// Create a source location from a position
    pub fn location_at(&self, position: usize) -> SourceLocation {
        let lines: Vec<&str> = self.source.lines().collect();
        let mut current_pos = 0;
        let mut line = 1;
        let mut column = 1;

        for (line_idx, line_content) in lines.iter().enumerate() {
            let line_end = current_pos + line_content.len();
            if position <= line_end {
                line = line_idx + 1;
                column = position - current_pos + 1;
                break;
            }
            current_pos = line_end + 1; // +1 for newline
        }

        // Handle case where position is at end of file
        if line == 0 && !lines.is_empty() {
            line = lines.len();
            column = lines.last().unwrap_or(&"").len() + 1;
        }

        SourceLocation {
            position,
            line,
            column,
        }
    }

    /// Get the source line at a given position
    pub fn source_line_at(&self, position: usize) -> String {
        let lines: Vec<&str> = self.source.lines().collect();
        let mut current_pos = 0;

        for line_content in lines.iter() {
            let line_end = current_pos + line_content.len();
            if position <= line_end {
                return line_content.to_string();
            }
            current_pos = line_end + 1; // +1 for newline
        }

        // Return last line if position is at end
        lines.last().unwrap_or(&"").to_string()
    }

    /// Create a lexer diagnostic
    pub fn lex_error(&self, position: usize, message: String) -> Diagnostic {
        let location = self.location_at(position);
        let source_line = self.source_line_at(position);
        
        Diagnostic::new(
            DiagnosticKind::LexError,
            location,
            message,
            source_line,
        )
    }

    /// Create a parser diagnostic
    pub fn parse_error(&self, position: usize, message: String) -> Diagnostic {
        let location = self.location_at(position);
        let source_line = self.source_line_at(position);
        
        Diagnostic::new(
            DiagnosticKind::ParseError,
            location,
            message,
            source_line,
        )
    }
}
