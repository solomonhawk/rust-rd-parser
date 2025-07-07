use thiserror::Error;

/// Represents the different types of errors that can occur during parsing
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParseError {
    #[error("Unexpected token: expected {expected}, found {found} at position {position}")]
    UnexpectedToken {
        expected: String,
        found: String,
        position: usize,
    },

    #[error("Unexpected end of input at position {position}")]
    UnexpectedEof { position: usize },

    #[error("Invalid character '{character}' at position {position}")]
    InvalidCharacter { character: char, position: usize },

    #[error("Invalid number format at position {position}")]
    InvalidNumber { position: usize },
}

/// Represents errors that can occur during lexical analysis
#[derive(Error, Debug, Clone, PartialEq)]
pub enum LexError {
    #[error("Invalid character '{character}' at position {position}")]
    InvalidCharacter { character: char, position: usize },

    #[error("Invalid number format at position {position}")]
    InvalidNumber { position: usize },
}

/// Result type for parsing operations
pub type ParseResult<T> = Result<T, ParseError>;

/// Result type for lexing operations
pub type LexResult<T> = Result<T, LexError>;

impl From<LexError> for ParseError {
    fn from(lex_error: LexError) -> Self {
        match lex_error {
            LexError::InvalidCharacter {
                character,
                position,
            } => ParseError::InvalidCharacter {
                character,
                position,
            },
            LexError::InvalidNumber { position } => ParseError::InvalidNumber { position },
        }
    }
}
