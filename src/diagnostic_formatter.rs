use crate::diagnostic::{Diagnostic, Severity};
use std::fmt;

/// Formats diagnostics into human-readable output
pub struct DiagnosticFormatter {
    use_colors: bool,
    show_suggestions: bool,
}

impl DiagnosticFormatter {
    pub fn new() -> Self {
        Self {
            use_colors: true,
            show_suggestions: true,
        }
    }

    pub fn with_colors(mut self, use_colors: bool) -> Self {
        self.use_colors = use_colors;
        self
    }

    pub fn with_suggestions(mut self, show_suggestions: bool) -> Self {
        self.show_suggestions = show_suggestions;
        self
    }

    /// Format a single diagnostic into a string
    pub fn format(&self, diagnostic: &Diagnostic) -> String {
        let mut output = String::new();

        // Error header with emoji
        let severity_icon = match diagnostic.severity() {
            Severity::Error => "❌",
            Severity::Warning => "⚠️",
            Severity::Info => "ℹ️",
            Severity::Hint => "💡",
        };

        output.push_str(&format!("{} {}\n", severity_icon, diagnostic.message));
        output.push_str(&format!(
            "    ┌─ line {}:{}\n",
            diagnostic.location.line, diagnostic.location.column
        ));
        output.push_str("    │\n");

        // Show the problematic line
        output.push_str(&format!(
            "{:3} │ {}\n",
            diagnostic.location.line, diagnostic.source_line
        ));

        // Show the error pointer
        let pointer_line = if let (Some(_end_position), Some(end_column)) = 
            (diagnostic.location.end_position, diagnostic.location.end_column) {
            // Span-based highlighting
            let start_col = diagnostic.location.column.saturating_sub(1);
            let span_length = end_column.saturating_sub(diagnostic.location.column).max(1);
            format!(
                "    │ {}{}",
                " ".repeat(start_col),
                "^".repeat(span_length)
            )
        } else {
            // Single position highlighting
            format!(
                "    │ {}^",
                " ".repeat(diagnostic.location.column.saturating_sub(1))
            )
        };
        output.push_str(&pointer_line);
        output.push('\n');

        // Add suggestion if provided and enabled
        if self.show_suggestions {
            if let Some(suggestion) = &diagnostic.suggestion {
                output.push_str("    │\n");
                output.push_str(&format!("    = 💡 suggestion: {}\n", suggestion));
            }
        }

        output
    }

    /// Format multiple diagnostics
    pub fn format_multiple(&self, diagnostics: &[Diagnostic]) -> String {
        diagnostics
            .iter()
            .map(|d| self.format(d))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for DiagnosticFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let formatter = DiagnosticFormatter::new();
        write!(f, "{}", formatter.format(self))
    }
}
