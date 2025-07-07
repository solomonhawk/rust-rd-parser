use std::fmt;

/// Enhanced error context for better error reporting
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorContext {
    pub source: String,
    pub position: usize,
    pub line: usize,
    pub column: usize,
    pub line_text: String,
}

impl ErrorContext {
    pub fn new(source: &str, position: usize) -> Self {
        let lines: Vec<&str> = source.lines().collect();
        let mut current_pos = 0;
        let mut line = 0;
        let mut column = 0;
        let mut line_text = String::new();

        for (line_idx, line_content) in lines.iter().enumerate() {
            let line_end = current_pos + line_content.len();
            if position <= line_end {
                line = line_idx + 1;
                column = position - current_pos + 1;
                line_text = line_content.to_string();
                break;
            }
            current_pos = line_end + 1; // +1 for newline
        }

        // Handle case where position is at end of file
        if line_text.is_empty() && !lines.is_empty() {
            line = lines.len();
            line_text = lines.last().unwrap_or(&"").to_string();
            column = line_text.len() + 1;
        }

        Self {
            source: source.to_string(),
            position,
            line,
            column,
            line_text,
        }
    }

    pub fn format_error_location(&self, message: &str, suggestion: Option<&str>) -> String {
        let mut output = String::new();

        // Error header
        output.push_str(&format!("❌ {}\n", message));
        output.push_str(&format!("   ┌─ line {}:{}\n", self.line, self.column));
        output.push_str(&format!("   │\n"));

        // Show the problematic line
        output.push_str(&format!("{:3} │ {}\n", self.line, self.line_text));

        // Show the error pointer
        let pointer_line = format!("   │ {}^", " ".repeat(self.column.saturating_sub(1)));
        output.push_str(&pointer_line);
        output.push('\n');

        // Add suggestion if provided
        if let Some(suggestion) = suggestion {
            output.push_str(&format!("   │\n"));
            output.push_str(&format!("   = 💡 suggestion: {}\n", suggestion));
        }

        output
    }
}

/// Represents the different types of errors that can occur during parsing
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedToken {
        expected: String,
        found: String,
        context: ErrorContext,
        suggestion: Option<String>,
    },

    UnexpectedEof {
        context: ErrorContext,
        expected: String,
    },

    InvalidCharacter {
        character: char,
        context: ErrorContext,
        suggestion: Option<String>,
    },

    InvalidNumber {
        context: ErrorContext,
        reason: String,
        suggestion: Option<String>,
    },
}

/// Represents errors that can occur during lexical analysis
#[derive(Debug, Clone, PartialEq)]
pub enum LexError {
    InvalidCharacter {
        character: char,
        context: ErrorContext,
        suggestion: Option<String>,
    },

    InvalidNumber {
        context: ErrorContext,
        reason: String,
        suggestion: Option<String>,
    },
}

/// Result type for parsing operations
pub type ParseResult<T> = Result<T, ParseError>;

/// Result type for lexing operations
pub type LexResult<T> = Result<T, LexError>;

impl ParseError {
    pub fn format_error(&self) -> String {
        match self {
            ParseError::UnexpectedToken {
                expected,
                found,
                context,
                suggestion,
            } => {
                let message = format!("Expected {}, but found {}", expected, found);
                context.format_error_location(&message, suggestion.as_deref())
            }
            ParseError::UnexpectedEof { context, expected } => {
                let message = format!("Unexpected end of input, expected {}", expected);
                let suggestion = "Make sure your input is complete";
                context.format_error_location(&message, Some(suggestion))
            }
            ParseError::InvalidCharacter {
                character,
                context,
                suggestion,
            } => {
                let message = format!("Invalid character '{}'", character);
                context.format_error_location(&message, suggestion.as_deref())
            }
            ParseError::InvalidNumber {
                context,
                reason,
                suggestion,
            } => {
                let message = format!("Invalid number: {}", reason);
                context.format_error_location(&message, suggestion.as_deref())
            }
        }
    }
}

impl LexError {
    pub fn format_error(&self) -> String {
        match self {
            LexError::InvalidCharacter {
                character,
                context,
                suggestion,
            } => {
                let message = format!("Invalid character '{}'", character);
                context.format_error_location(&message, suggestion.as_deref())
            }
            LexError::InvalidNumber {
                context,
                reason,
                suggestion,
            } => {
                let message = format!("Invalid number: {}", reason);
                context.format_error_location(&message, suggestion.as_deref())
            }
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_error())
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_error())
    }
}

impl From<LexError> for ParseError {
    fn from(lex_error: LexError) -> Self {
        match lex_error {
            LexError::InvalidCharacter {
                character,
                context,
                suggestion,
            } => ParseError::InvalidCharacter {
                character,
                context,
                suggestion,
            },
            LexError::InvalidNumber {
                context,
                reason,
                suggestion,
            } => ParseError::InvalidNumber {
                context,
                reason,
                suggestion,
            },
        }
    }
}
