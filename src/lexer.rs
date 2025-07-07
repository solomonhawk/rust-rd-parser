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

    /// Text segment within rule content (between expressions)
    TextSegment(String),

    /// Hash symbol '#' for table declarations
    Hash,

    /// Table identifier (after #)
    Identifier(String),

    /// Modifier keyword for table references
    Modifier(String),

    /// Dice roll expression (like "d6", "2d10")
    DiceRoll { count: Option<u32>, sides: u32 },

    /// Left bracket '['
    LeftBracket,

    /// Right bracket ']'
    RightBracket,

    /// Left curly brace '{'
    LeftBrace,

    /// Right curly brace '}'
    RightBrace,

    /// Export keyword
    Export,

    /// Pipe separator '|' for modifiers
    Pipe,

    /// At symbol '@' for external references
    At,

    /// Forward slash '/' for external references
    Slash,

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
    in_expression: bool,
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
            in_expression: false,
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

            // Handle comments and forward slash
            '/' => {
                if self.peek() == '/' {
                    // Line comment - consume until end of line
                    self.line_comment()
                } else if self.peek() == '*' {
                    // Block comment - consume until */
                    self.block_comment()
                } else if self.in_expression {
                    // Forward slash in expression (for external references like @user/collection)
                    Ok(Some(self.make_token(TokenType::Slash)))
                } else if self.in_rule_text && !self.in_expression {
                    // Regular '/' character in rule text
                    self.current -= 1;
                    self.text_segment()
                } else {
                    // Invalid '/' character outside rule text
                    let diagnostic = self
                        .diagnostic_collector
                        .lex_error(self.current - 1, "Invalid character '/'".to_string())
                        .with_suggestion(
                            "Only numbers, colons, and rule text are allowed in this language"
                                .to_string(),
                        );

                    Err(LexError::InvalidCharacter {
                        character: c,
                        diagnostic: Box::new(diagnostic),
                    })
                }
            }

            // At symbol for external references (only in expressions)
            '@' if self.in_expression => Ok(Some(self.make_token(TokenType::At))),

            // Newlines end rule text and reset state
            '\n' => {
                self.in_rule_text = false;
                Ok(Some(self.make_token(TokenType::Newline)))
            }

            // Hash symbol for table declarations or expressions
            '#' if !self.in_rule_text || self.in_expression => {
                Ok(Some(self.make_token(TokenType::Hash)))
            }

            // Left bracket for flags
            '[' if !self.in_rule_text => Ok(Some(self.make_token(TokenType::LeftBracket))),

            // Right bracket for flags
            ']' if !self.in_rule_text => Ok(Some(self.make_token(TokenType::RightBracket))),

            // Left brace for expressions (can appear in rule text)
            '{' => {
                self.in_expression = true;
                Ok(Some(self.make_token(TokenType::LeftBrace)))
            }

            // Right brace for expressions (can appear in rule text)
            '}' => {
                self.in_expression = false;
                Ok(Some(self.make_token(TokenType::RightBrace)))
            }

            // Pipe separator for modifiers (only in expressions)
            '|' if self.in_expression => Ok(Some(self.make_token(TokenType::Pipe))),

            // Colon transitions us into rule content mode
            ':' if !self.in_rule_text => {
                self.in_rule_text = true;
                Ok(Some(self.make_token(TokenType::Colon)))
            }

            // Numbers (positive floating point only) - only when not in rule text
            c if c.is_ascii_digit() && !self.in_rule_text => self.number(),

            // Dice rolls or identifiers when in expressions
            c if (c.is_alphabetic() || c.is_ascii_digit()) && self.in_expression => {
                // Check if this might be a dice roll
                if c == 'd' && !self.is_at_end() && self.peek().is_ascii_digit() {
                    // This is a dice roll starting with 'd'
                    self.dice_roll()
                } else if c.is_ascii_digit() && self.peek_for_dice() {
                    // This is a dice roll starting with a number
                    self.dice_roll()
                } else {
                    // Regular identifier
                    self.identifier()
                }
            }

            // Identifiers (table names and keywords) - allowed outside rule text
            c if c.is_alphabetic() && !self.in_rule_text => self.identifier(),

            // Text content when in rule text mode but not in expression
            _ if self.in_rule_text && !self.in_expression && c != '{' && c != '}' && c != '\n' => {
                // Backtrack and collect text segment
                self.current -= 1;
                self.text_segment()
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

    fn identifier(&mut self) -> LexResult<Option<Token>> {
        // Collect alphanumeric characters, underscores, and hyphens
        while !self.is_at_end()
            && (self.peek().is_alphanumeric() || self.peek() == '_' || self.peek() == '-')
        {
            self.advance();
        }

        let text = self.lexeme();
        let token_type = match text.as_str() {
            "export" => TokenType::Export,
            // Check if this is a known modifier keyword
            "indefinite" | "definite" | "capitalize" | "uppercase" | "lowercase" => {
                TokenType::Modifier(text.clone())
            }
            // All other identifiers (including unknown modifiers) become regular identifiers
            _ => TokenType::Identifier(text.clone()),
        };

        Ok(Some(Token::new(
            token_type,
            text,
            Span::new(self.start, self.current),
        )))
    }

    fn text_segment(&mut self) -> LexResult<Option<Token>> {
        // Don't skip whitespace - we want to preserve spaces between expressions
        // Collect text until we hit a brace, newline, comment, or EOF
        while !self.is_at_end()
            && self.peek() != '{'
            && self.peek() != '}'
            && self.peek() != '\n'
            && !(self.peek() == '/' && (self.peek_next() == '/' || self.peek_next() == '*'))
        {
            self.advance();
        }

        let text = self.lexeme();

        if text.is_empty() {
            return Ok(None); // Skip empty text segments
        }

        Ok(Some(Token::new(
            TokenType::TextSegment(text.clone()),
            text.clone(),
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

    fn line_comment(&mut self) -> LexResult<Option<Token>> {
        // Consume the second '/'
        self.advance();

        // Consume characters until end of line or end of file
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }

        // Return None to skip this comment
        Ok(None)
    }

    fn block_comment(&mut self) -> LexResult<Option<Token>> {
        // Consume the '*'
        self.advance();

        // Look for the closing */
        while !self.is_at_end() {
            if self.peek() == '*' && self.peek_next() == '/' {
                // Found the end - consume both characters
                self.advance(); // consume '*'
                self.advance(); // consume '/'
                break;
            }

            // If we encounter a newline, reset rule text state
            if self.peek() == '\n' {
                self.in_rule_text = false;
            }

            self.advance();
        }

        // Check if we reached EOF without finding the closing */
        if self.is_at_end()
            && !(self.input.len() >= 2
                && self.input[self.input.len() - 2] == '*'
                && self.input[self.input.len() - 1] == '/')
        {
            let diagnostic = self
                .diagnostic_collector
                .lex_error(self.start, "Unterminated block comment".to_string())
                .with_suggestion("Add */ to close the block comment".to_string());

            return Err(LexError::InvalidCharacter {
                character: '*',
                diagnostic: Box::new(diagnostic),
            });
        }

        // Return None to skip this comment
        Ok(None)
    }

    fn peek_for_dice(&self) -> bool {
        // Look ahead to see if this looks like a dice roll pattern
        let mut pos = self.current;

        // Skip digits (the count part)
        while pos < self.input.len() && self.input[pos].is_ascii_digit() {
            pos += 1;
        }

        // Check if we find a 'd' character
        pos < self.input.len() && self.input[pos] == 'd'
    }

    fn dice_roll(&mut self) -> LexResult<Option<Token>> {
        let mut count = None;

        // Check if we start with digits (the count) or 'd'
        let current_char = self.input[self.current - 1];

        if current_char.is_ascii_digit() {
            // Back up to parse the number
            self.current -= 1;
            let start_pos = self.current;

            // Parse the count
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                self.advance();
            }

            let count_str: String = self.input[start_pos..self.current].iter().collect();
            count = Some(count_str.parse::<u32>().map_err(|_| {
                let diagnostic = self
                    .diagnostic_collector
                    .lex_error(start_pos, format!("Invalid dice count: {}", count_str))
                    .with_suggestion(
                        "Dice count should be a positive integer like 2, 10, or 100".to_string(),
                    );

                LexError::InvalidNumber {
                    reason: format!("Invalid dice count: {}", count_str),
                    diagnostic: Box::new(diagnostic),
                }
            })?);
        } else if current_char == 'd' {
            // We start with 'd', no count specified (defaults to 1)
            // The 'd' is already consumed, so we continue to parse sides
        } else {
            // This shouldn't happen given our calling logic
            let diagnostic = self
                .diagnostic_collector
                .lex_error(
                    self.current - 1,
                    "Expected digit or 'd' in dice roll".to_string(),
                )
                .with_suggestion("Dice rolls should start with a number or 'd'".to_string());

            return Err(LexError::InvalidCharacter {
                character: current_char,
                diagnostic: Box::new(diagnostic),
            });
        }

        // Expect 'd' character (unless we already started with it)
        if current_char != 'd' {
            if !self.is_at_end() && self.peek() == 'd' {
                self.advance(); // consume 'd'
            } else {
                let diagnostic = self
                    .diagnostic_collector
                    .lex_error(
                        self.current,
                        "Expected 'd' in dice roll expression".to_string(),
                    )
                    .with_suggestion(
                        "Dice rolls should be formatted like 'd6', '2d10', or '100d20'".to_string(),
                    );

                return Err(LexError::InvalidCharacter {
                    character: self.peek(),
                    diagnostic: Box::new(diagnostic),
                });
            }
        }

        // Parse the sides (number of sides on the dice)
        let sides_start = self.current;
        while !self.is_at_end() && self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.current == sides_start {
            let diagnostic = self
                .diagnostic_collector
                .lex_error(
                    self.current,
                    "Expected number of sides after 'd'".to_string(),
                )
                .with_suggestion(
                    "Dice rolls should specify the number of sides like 'd6', 'd10', or 'd20'"
                        .to_string(),
                );

            return Err(LexError::InvalidCharacter {
                character: self.peek(),
                diagnostic: Box::new(diagnostic),
            });
        }

        let sides_str: String = self.input[sides_start..self.current].iter().collect();
        let sides = sides_str.parse::<u32>().map_err(|_| {
            let diagnostic = self
                .diagnostic_collector
                .lex_error(sides_start, format!("Invalid dice sides: {}", sides_str))
                .with_suggestion(
                    "Dice sides should be a positive integer like 6, 10, or 20".to_string(),
                );

            LexError::InvalidNumber {
                reason: format!("Invalid dice sides: {}", sides_str),
                diagnostic: Box::new(diagnostic),
            }
        })?;

        if sides == 0 {
            let diagnostic = self
                .diagnostic_collector
                .lex_error(sides_start, "Dice must have at least 1 side".to_string())
                .with_suggestion(
                    "Use positive numbers for dice sides like 6, 10, or 20".to_string(),
                );

            return Err(LexError::InvalidNumber {
                reason: "Dice must have at least 1 side".to_string(),
                diagnostic: Box::new(diagnostic),
            });
        }

        Ok(Some(Token::new(
            TokenType::DiceRoll { count, sides },
            self.lexeme(),
            Span::new(self.start, self.current),
        )))
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::Number(n) => write!(f, "{}", n),
            TokenType::Colon => write!(f, ":"),
            TokenType::RuleText(text) => write!(f, "{}", text),
            TokenType::TextSegment(text) => write!(f, "{}", text),
            TokenType::Hash => write!(f, "#"),
            TokenType::Identifier(name) => write!(f, "{}", name),
            TokenType::Modifier(name) => write!(f, "{}", name),
            TokenType::DiceRoll { count, sides } => match count {
                Some(c) => write!(f, "{}d{}", c, sides),
                None => write!(f, "d{}", sides),
            },
            TokenType::LeftBracket => write!(f, "["),
            TokenType::RightBracket => write!(f, "]"),
            TokenType::LeftBrace => write!(f, "{{"),
            TokenType::RightBrace => write!(f, "}}"),
            TokenType::Export => write!(f, "export"),
            TokenType::Pipe => write!(f, "|"),
            TokenType::At => write!(f, "@"),
            TokenType::Slash => write!(f, "/"),
            TokenType::Newline => write!(f, "\\n"),
            TokenType::Eof => write!(f, "EOF"),
        }
    }
}
