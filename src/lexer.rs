use crate::ast::Span;
use crate::diagnostic_collector::DiagnosticCollector;
use crate::errors::{LexError, LexResult};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Represents the different types of tokens in our TBL language
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TokenType {
    /// A positive floating point number
    Number(f64),

    /// The colon separator ':'
    Colon,

    /// Rule text (everything after the colon until newline)
    RuleText(String),

    /// Hash symbol '#' for table declarations
    Hash,

    /// Table identifier (after #)
    Identifier(String),

    /// Left bracket '['
    LeftBracket,

    /// Right bracket ']'
    RightBracket,

    /// Export keyword
    Export,

    /// Newline character
    Newline,

    /// End of file
    Eof,
}

/// A token with its type, lexeme, and position
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub span: Span,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, span: Span) -> Self {
        Self {
            token_type,
            lexeme,
            span,
        }
    }
}

/// Lexer for tokenizing input source code
pub struct Lexer {
    input: Vec<char>,
    current: usize,
    start: usize,
    in_rule_text: bool,
    diagnostic_collector: DiagnosticCollector,
}

impl Lexer {
    /// Creates a new lexer for the given input
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            current: 0,
            start: 0,
            in_rule_text: false,
            diagnostic_collector: DiagnosticCollector::new(input.to_string()),
        }
    }

    /// Tokenizes the entire input and returns a vector of tokens
    pub fn tokenize(&mut self) -> LexResult<Vec<Token>> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.start = self.current;
            match self.scan_token() {
                Ok(Some(token)) => {
                    tokens.push(token);
                }
                Ok(None) => {} // Skip whitespace (except newlines)
                Err(e) => return Err(e),
            }
        }

        tokens.push(Token::new(
            TokenType::Eof,
            String::new(),
            Span::new(self.current, self.current),
        ));

        Ok(tokens)
    }

    fn scan_token(&mut self) -> LexResult<Option<Token>> {
        let c = self.advance();

        match c {
            // Skip spaces and tabs (except when in rule text)
            ' ' | '\t' if !self.in_rule_text => Ok(None),

            // Newlines end rule text and reset state
            '\n' => {
                self.in_rule_text = false;
                Ok(Some(self.make_token(TokenType::Newline)))
            }

            // Hash symbol for table declarations
            '#' if !self.in_rule_text => Ok(Some(self.make_token(TokenType::Hash))),

            // Left bracket for flags
            '[' if !self.in_rule_text => Ok(Some(self.make_token(TokenType::LeftBracket))),

            // Right bracket for flags
            ']' if !self.in_rule_text => Ok(Some(self.make_token(TokenType::RightBracket))),

            // Colon transitions us into rule text mode
            ':' if !self.in_rule_text => {
                self.in_rule_text = true;
                Ok(Some(self.make_token(TokenType::Colon)))
            }

            // Numbers (positive floating point only) - only when not in rule text
            c if c.is_ascii_digit() && !self.in_rule_text => self.number(),

            // Identifiers (table names and keywords) - only when not in rule text
            c if c.is_alphabetic() && !self.in_rule_text => self.identifier(),

            // Everything else when in rule text mode
            _ if self.in_rule_text => {
                // Backtrack and collect rule text
                self.current -= 1;
                self.rule_text()
            }

            _ => {
                let suggestion = match c {
                    '-' => Some(
                        "Negative numbers are not allowed. Use positive weights like 1.0, 2.5"
                            .to_string(),
                    ),
                    ':' => Some("Colons are only allowed after a weight number".to_string()),
                    _ => Some(
                        "Only numbers, colons, and rule text are allowed in this language"
                            .to_string(),
                    ),
                };

                let diagnostic = self
                    .diagnostic_collector
                    .lex_error(self.current - 1, format!("Invalid character '{}'", c))
                    .with_suggestion(suggestion.unwrap());

                Err(LexError::InvalidCharacter {
                    character: c,
                    diagnostic: Box::new(diagnostic),
                })
            }
        }
    }

    fn number(&mut self) -> LexResult<Option<Token>> {
        // Parse integer part
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        // Look for decimal part
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance(); // consume '.'

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let lexeme = self.lexeme();
        let value = lexeme.parse::<f64>().map_err(|_| {
            let diagnostic = self
                .diagnostic_collector
                .lex_error(self.start, format!("'{}' is not a valid number", lexeme))
                .with_suggestion(
                    "Numbers should be positive decimal values like 1.5, 2.0, or 42".to_string(),
                );

            LexError::InvalidNumber {
                reason: format!("'{}' is not a valid number", lexeme),
                diagnostic: Box::new(diagnostic),
            }
        })?;

        // Ensure it's positive
        if value <= 0.0 {
            let diagnostic = self
                .diagnostic_collector
                .lex_error(
                    self.start,
                    format!("Weight must be positive, but got {}", value),
                )
                .with_suggestion("Try using a positive number like 1.0, 2.5, or 10".to_string());

            return Err(LexError::InvalidNumber {
                reason: format!("Weight must be positive, but got {}", value),
                diagnostic: Box::new(diagnostic),
            });
        }

        Ok(Some(Token::new(
            TokenType::Number(value),
            self.lexeme(),
            Span::new(self.start, self.current),
        )))
    }

    fn rule_text(&mut self) -> LexResult<Option<Token>> {
        // Skip leading whitespace
        while !self.is_at_end() && (self.peek() == ' ' || self.peek() == '\t') {
            self.advance();
        }

        self.start = self.current; // Reset start after skipping whitespace

        // Collect everything until newline or EOF
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }

        let text = self.lexeme().trim_end().to_string();

        if text.is_empty() {
            return Ok(None); // Skip empty rule text
        }

        Ok(Some(Token::new(
            TokenType::RuleText(text.clone()),
            text,
            Span::new(self.start, self.current),
        )))
    }

    fn identifier(&mut self) -> LexResult<Option<Token>> {
        // Collect alphanumeric characters and underscores
        while !self.is_at_end() && (self.peek().is_alphanumeric() || self.peek() == '_') {
            self.advance();
        }

        let text = self.lexeme();
        let token_type = match text.as_str() {
            "export" => TokenType::Export,
            _ => TokenType::Identifier(text.clone()),
        };

        Ok(Some(Token::new(
            token_type,
            text,
            Span::new(self.start, self.current),
        )))
    }

    // Helper methods
    fn advance(&mut self) -> char {
        self.current += 1;
        self.input[self.current - 1]
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.input[self.current]
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.input.len() {
            '\0'
        } else {
            self.input[self.current + 1]
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.input.len()
    }

    fn lexeme(&self) -> String {
        self.input[self.start..self.current].iter().collect()
    }

    fn make_token(&self, token_type: TokenType) -> Token {
        Token::new(
            token_type,
            self.lexeme(),
            Span::new(self.start, self.current),
        )
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::Number(n) => write!(f, "{}", n),
            TokenType::Colon => write!(f, ":"),
            TokenType::RuleText(text) => write!(f, "{}", text),
            TokenType::Hash => write!(f, "#"),
            TokenType::Identifier(name) => write!(f, "{}", name),
            TokenType::LeftBracket => write!(f, "["),
            TokenType::RightBracket => write!(f, "]"),
            TokenType::Export => write!(f, "export"),
            TokenType::Newline => write!(f, "\\n"),
            TokenType::Eof => write!(f, "EOF"),
        }
    }
}
