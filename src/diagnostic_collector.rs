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
            end_position: None,
            end_column: None,
        }
    }

    /// Create a source location from a span (start to end positions)
    pub fn location_span(&self, start_position: usize, end_position: usize) -> SourceLocation {
        let lines: Vec<&str> = self.source.lines().collect();
        let mut current_pos = 0;
        let mut start_line = 1;
        let mut start_column = 1;
        let mut end_column = 1;

        // Find start position
        for (line_idx, line_content) in lines.iter().enumerate() {
            let line_end = current_pos + line_content.len();
            if start_position <= line_end {
                start_line = line_idx + 1;
                start_column = start_position - current_pos + 1;
                
                // Calculate end column on the same line
                if end_position <= line_end {
                    end_column = end_position - current_pos + 1;
                } else {
                    end_column = line_content.len() + 1;
                }
                break;
            }
            current_pos = line_end + 1; // +1 for newline
        }

        // Handle case where position is at end of file
        if start_line == 0 && !lines.is_empty() {
            start_line = lines.len();
            start_column = lines.last().unwrap_or(&"").len() + 1;
        }

        SourceLocation {
            position: start_position,
            line: start_line,
            column: start_column,
            end_position: Some(end_position),
            end_column: Some(end_column),
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

    /// Create a parser diagnostic with span highlighting
    pub fn parse_error_span(&self, start_position: usize, end_position: usize, message: String) -> Diagnostic {
        let location = self.location_span(start_position, end_position);
        let source_line = self.source_line_at(start_position);
        
        Diagnostic::new(
            DiagnosticKind::ParseError,
            location,
            message,
            source_line,
        )
    }
}
