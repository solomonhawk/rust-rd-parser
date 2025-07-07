use crate::ast::{Node, Program, Rule, Span};
use crate::errors::{ErrorContext, ParseError, ParseResult};
use crate::lexer::{Token, TokenType};

/// Simple parser for our weight: rule language
pub struct Parser {
    tokens: Vec<Token>,
    source: String,
    current: usize,
}

impl Parser {
    /// Creates a new parser with the given tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            source: String::new(), // Will be set by from_source
            current: 0,
        }
    }

    /// Creates a new parser with tokens and source for better error reporting
    pub fn from_source(tokens: Vec<Token>, source: String) -> Self {
        Self {
            tokens,
            source,
            current: 0,
        }
    }

    /// Parses the tokens into an AST
    pub fn parse(&mut self) -> ParseResult<Program> {
        let mut rules = Vec::new();

        while !self.is_at_end() {
            // Skip newlines at the top level
            if self.check(&TokenType::Newline) {
                self.advance();
                continue;
            }

            rules.push(self.rule()?);
        }

        Ok(Program::new(rules))
    }

    /// Parses a single rule: weight: rule_text
    fn rule(&mut self) -> ParseResult<Node<Rule>> {
        let start_pos = self.peek().span.start;

        // Expect a number (weight)
        let weight = if let TokenType::Number(n) = &self.advance().token_type {
            *n
        } else {
            let token = self.previous();
            let context = ErrorContext::new(&self.source, token.span.start);
            let suggestion = match &token.token_type {
                TokenType::RuleText(_) => Some("Rules must start with a weight. Try adding a number like '1.0:' before the rule text".to_string()),
                TokenType::Colon => Some("Missing weight before colon. Try adding a number like '1.0' before the ':'".to_string()),
                TokenType::Eof => Some("File ended unexpectedly. Add a weight and rule like '1.0: some rule'".to_string()),
                _ => Some("Expected a positive number (weight) at the start of each rule".to_string()),
            };

            return Err(ParseError::UnexpectedToken {
                expected: "positive number (weight)".to_string(),
                found: format!("{}", token.token_type),
                context,
                suggestion,
            });
        };

        // Expect a colon
        self.consume(&TokenType::Colon, "Expected ':' after weight")?;

        // Expect rule text
        let text = if let TokenType::RuleText(text) = &self.advance().token_type {
            text.clone()
        } else {
            let token = self.previous();
            let context = ErrorContext::new(&self.source, token.span.start);
            let suggestion = match &token.token_type {
                TokenType::Newline => Some("Missing rule text after colon. Add some text describing the rule".to_string()),
                TokenType::Eof => Some("File ended after colon. Add rule text like 'some rule description'".to_string()),
                TokenType::Number(_) => Some("Found another number when expecting rule text. Each rule should have format 'weight: text'".to_string()),
                _ => Some("Expected rule text after the colon".to_string()),
            };

            return Err(ParseError::UnexpectedToken {
                expected: "rule text after colon".to_string(),
                found: format!("{}", token.token_type),
                context,
                suggestion,
            });
        };

        // Optional newline
        if self.check(&TokenType::Newline) {
            self.advance();
        }

        let end_pos = self.previous().span.end;
        let rule = Rule { weight, text };

        Ok(Node::new(rule, Span::new(start_pos, end_pos)))
    }

    // Utility methods
    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.peek().token_type) == std::mem::discriminant(token_type)
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().token_type, TokenType::Eof)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) -> ParseResult<&Token> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            let token = self.peek();
            let context = ErrorContext::new(&self.source, token.span.start);
            let suggestion = match (&token.token_type, token_type) {
                (TokenType::RuleText(_), TokenType::Colon) => Some(
                    "Missing colon after weight. Add ':' between the weight and rule text"
                        .to_string(),
                ),
                (TokenType::Number(_), TokenType::Colon) => {
                    Some("Missing colon after weight. Add ':' after the number".to_string())
                }
                (TokenType::Eof, _) => {
                    Some("File ended unexpectedly. Complete the current rule".to_string())
                }
                _ => Some(format!("Expected {}", message)),
            };

            Err(ParseError::UnexpectedToken {
                expected: message.to_string(),
                found: format!("{}", token.token_type),
                context,
                suggestion,
            })
        }
    }
}
