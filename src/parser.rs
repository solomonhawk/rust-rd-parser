use crate::ast::{Node, Program, Rule, Span};
use crate::errors::{ParseError, ParseResult};
use crate::lexer::{Token, TokenType};

/// Simple parser for our weight: rule language
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    /// Creates a new parser with the given tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
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
            return Err(ParseError::UnexpectedToken {
                expected: "positive number".to_string(),
                found: format!("{}", self.previous().token_type),
                position: self.previous().span.start,
            });
        };
        
        // Expect a colon
        self.consume(&TokenType::Colon, "Expected ':' after weight")?;
        
        // Expect rule text
        let text = if let TokenType::RuleText(text) = &self.advance().token_type {
            text.clone()
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "rule text".to_string(),
                found: format!("{}", self.previous().token_type),
                position: self.previous().span.start,
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
            Err(ParseError::UnexpectedToken {
                expected: message.to_string(),
                found: format!("{}", self.peek().token_type),
                position: self.peek().span.start,
            })
        }
    }
}