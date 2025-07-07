use crate::diagnostic::Diagnostic;
use std::fmt;

/// Represents lexical analysis errors with diagnostic information
#[derive(Debug, Clone, PartialEq)]
pub enum LexError {
    InvalidCharacter {
        character: char,
        diagnostic: Box<Diagnostic>,
    },
    InvalidNumber {
        reason: String,
        diagnostic: Box<Diagnostic>,
    },
}

/// Represents parsing errors with diagnostic information
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedToken {
        expected: String,
        found: String,
        diagnostic: Box<Diagnostic>,
    },
    UnexpectedEof {
        expected: String,
        diagnostic: Box<Diagnostic>,
    },
    InvalidCharacter {
        character: char,
        diagnostic: Box<Diagnostic>,
    },
    InvalidNumber {
        reason: String,
        diagnostic: Box<Diagnostic>,
    },
}

/// Result type for parsing operations
pub type ParseResult<T> = Result<T, ParseError>;

/// Result type for lexing operations
pub type LexResult<T> = Result<T, LexError>;

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexError::InvalidCharacter { diagnostic, .. } => write!(f, "{}", diagnostic),
            LexError::InvalidNumber { diagnostic, .. } => write!(f, "{}", diagnostic),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedToken { diagnostic, .. } => write!(f, "{}", diagnostic),
            ParseError::UnexpectedEof { diagnostic, .. } => write!(f, "{}", diagnostic),
            ParseError::InvalidCharacter { diagnostic, .. } => write!(f, "{}", diagnostic),
            ParseError::InvalidNumber { diagnostic, .. } => write!(f, "{}", diagnostic),
        }
    }
}

impl From<LexError> for ParseError {
    fn from(lex_error: LexError) -> Self {
        match lex_error {
            LexError::InvalidCharacter {
                character,
                diagnostic,
            } => ParseError::InvalidCharacter {
                character,
                diagnostic,
            },
            LexError::InvalidNumber { reason, diagnostic } => {
                ParseError::InvalidNumber { reason, diagnostic }
            }
        }
    }
}

impl std::error::Error for LexError {}
impl std::error::Error for ParseError {}
