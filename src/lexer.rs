use crate::ast::Span;
use crate::errors::{LexError, LexResult};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Represents the different types of tokens in our language
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TokenType {
    // Literals
    Number(f64),
    String(String),
    Identifier(String),

    // Keywords
    True,
    False,
    Null,
    Let,
    If,
    Else,
    While,
    Return,
    Function,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    AndAnd,
    OrOr,

    // Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Semicolon,

    // Special
    Newline,
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
}

impl Lexer {
    /// Creates a new lexer for the given input
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            current: 0,
            start: 0,
        }
    }

    /// Tokenizes the entire input and returns a vector of tokens
    pub fn tokenize(&mut self) -> LexResult<Vec<Token>> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.start = self.current;
            match self.scan_token() {
                Ok(Some(token)) => tokens.push(token),
                Ok(None) => {} // Skip whitespace
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
            // Whitespace (skip)
            ' ' | '\r' | '\t' => Ok(None),

            // Newlines
            '\n' => Ok(Some(self.make_token(TokenType::Newline))),

            // Single-character tokens
            '(' => Ok(Some(self.make_token(TokenType::LeftParen))),
            ')' => Ok(Some(self.make_token(TokenType::RightParen))),
            '{' => Ok(Some(self.make_token(TokenType::LeftBrace))),
            '}' => Ok(Some(self.make_token(TokenType::RightBrace))),
            '[' => Ok(Some(self.make_token(TokenType::LeftBracket))),
            ']' => Ok(Some(self.make_token(TokenType::RightBracket))),
            ',' => Ok(Some(self.make_token(TokenType::Comma))),
            ';' => Ok(Some(self.make_token(TokenType::Semicolon))),
            '+' => Ok(Some(self.make_token(TokenType::Plus))),
            '-' => Ok(Some(self.make_token(TokenType::Minus))),
            '*' => Ok(Some(self.make_token(TokenType::Star))),
            '/' => Ok(Some(self.make_token(TokenType::Slash))),
            '%' => Ok(Some(self.make_token(TokenType::Percent))),

            // Two-character tokens
            '!' => {
                if self.match_char('=') {
                    Ok(Some(self.make_token(TokenType::BangEqual)))
                } else {
                    Ok(Some(self.make_token(TokenType::Bang)))
                }
            }
            '=' => {
                if self.match_char('=') {
                    Ok(Some(self.make_token(TokenType::EqualEqual)))
                } else {
                    Ok(Some(self.make_token(TokenType::Equal)))
                }
            }
            '<' => {
                if self.match_char('=') {
                    Ok(Some(self.make_token(TokenType::LessEqual)))
                } else {
                    Ok(Some(self.make_token(TokenType::Less)))
                }
            }
            '>' => {
                if self.match_char('=') {
                    Ok(Some(self.make_token(TokenType::GreaterEqual)))
                } else {
                    Ok(Some(self.make_token(TokenType::Greater)))
                }
            }
            '&' => {
                if self.match_char('&') {
                    Ok(Some(self.make_token(TokenType::AndAnd)))
                } else {
                    Err(LexError::InvalidCharacter {
                        character: c,
                        position: self.current - 1,
                    })
                }
            }
            '|' => {
                if self.match_char('|') {
                    Ok(Some(self.make_token(TokenType::OrOr)))
                } else {
                    Err(LexError::InvalidCharacter {
                        character: c,
                        position: self.current - 1,
                    })
                }
            }

            // String literals
            '"' => self.string(),

            // Numbers
            c if c.is_ascii_digit() => self.number(),

            // Identifiers and keywords
            c if c.is_alphabetic() || c == '_' => self.identifier(),

            _ => Err(LexError::InvalidCharacter {
                character: c,
                position: self.current - 1,
            }),
        }
    }

    fn string(&mut self) -> LexResult<Option<Token>> {
        let mut value = String::new();

        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                // Allow multiline strings
            }

            if self.peek() == '\\' {
                self.advance(); // consume backslash
                match self.peek() {
                    'n' => {
                        value.push('\n');
                        self.advance();
                    }
                    't' => {
                        value.push('\t');
                        self.advance();
                    }
                    'r' => {
                        value.push('\r');
                        self.advance();
                    }
                    '\\' => {
                        value.push('\\');
                        self.advance();
                    }
                    '"' => {
                        value.push('"');
                        self.advance();
                    }
                    c => {
                        return Err(LexError::InvalidEscape {
                            sequence: c,
                            position: self.current,
                        });
                    }
                }
            } else {
                value.push(self.advance());
            }
        }

        if self.is_at_end() {
            return Err(LexError::UnterminatedString {
                position: self.start,
            });
        }

        // Consume closing quote
        self.advance();

        Ok(Some(Token::new(
            TokenType::String(value),
            self.lexeme(),
            Span::new(self.start, self.current),
        )))
    }

    fn number(&mut self) -> LexResult<Option<Token>> {
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

        let value = self
            .lexeme()
            .parse::<f64>()
            .map_err(|_| LexError::InvalidNumber {
                position: self.start,
            })?;

        Ok(Some(Token::new(
            TokenType::Number(value),
            self.lexeme(),
            Span::new(self.start, self.current),
        )))
    }

    fn identifier(&mut self) -> LexResult<Option<Token>> {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let text = self.lexeme();
        let token_type = match text.as_str() {
            "true" => TokenType::True,
            "false" => TokenType::False,
            "null" => TokenType::Null,
            "let" => TokenType::Let,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "while" => TokenType::While,
            "return" => TokenType::Return,
            "function" => TokenType::Function,
            _ => TokenType::Identifier(text.clone()),
        };

        Ok(Some(Token::new(
            token_type,
            text,
            Span::new(self.start, self.current),
        )))
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.input[self.current - 1]
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.input[self.current] != expected {
            false
        } else {
            self.current += 1;
            true
        }
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
            TokenType::String(s) => write!(f, "\"{}\"", s),
            TokenType::Identifier(name) => write!(f, "{}", name),
            TokenType::True => write!(f, "true"),
            TokenType::False => write!(f, "false"),
            TokenType::Null => write!(f, "null"),
            TokenType::Let => write!(f, "let"),
            TokenType::If => write!(f, "if"),
            TokenType::Else => write!(f, "else"),
            TokenType::While => write!(f, "while"),
            TokenType::Return => write!(f, "return"),
            TokenType::Function => write!(f, "function"),
            TokenType::Plus => write!(f, "+"),
            TokenType::Minus => write!(f, "-"),
            TokenType::Star => write!(f, "*"),
            TokenType::Slash => write!(f, "/"),
            TokenType::Percent => write!(f, "%"),
            TokenType::Bang => write!(f, "!"),
            TokenType::BangEqual => write!(f, "!="),
            TokenType::Equal => write!(f, "="),
            TokenType::EqualEqual => write!(f, "=="),
            TokenType::Greater => write!(f, ">"),
            TokenType::GreaterEqual => write!(f, ">="),
            TokenType::Less => write!(f, "<"),
            TokenType::LessEqual => write!(f, "<="),
            TokenType::AndAnd => write!(f, "&&"),
            TokenType::OrOr => write!(f, "||"),
            TokenType::LeftParen => write!(f, "("),
            TokenType::RightParen => write!(f, ")"),
            TokenType::LeftBrace => write!(f, "{{"),
            TokenType::RightBrace => write!(f, "}}"),
            TokenType::LeftBracket => write!(f, "["),
            TokenType::RightBracket => write!(f, "]"),
            TokenType::Comma => write!(f, ","),
            TokenType::Semicolon => write!(f, ";"),
            TokenType::Newline => write!(f, "\\n"),
            TokenType::Eof => write!(f, "EOF"),
        }
    }
}
