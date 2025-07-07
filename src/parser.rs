use crate::ast::{Node, Program, Rule, Span, Table, TableMetadata};
use crate::diagnostic_collector::DiagnosticCollector;
use crate::errors::{ParseError, ParseResult};
use crate::lexer::{Token, TokenType};

/// Simple parser for our weight: rule language
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    diagnostic_collector: DiagnosticCollector,
}

impl Parser {
    /// Creates a new parser with the given tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            diagnostic_collector: DiagnosticCollector::new(String::new()),
        }
    }

    /// Creates a new parser with tokens and source for better error reporting
    pub fn from_source(tokens: Vec<Token>, source: String) -> Self {
        Self {
            tokens,
            current: 0,
            diagnostic_collector: DiagnosticCollector::new(source),
        }
    }

    /// Parses the tokens into an AST containing tables
    pub fn parse(&mut self) -> ParseResult<Program> {
        let mut tables = Vec::new();

        while !self.is_at_end() {
            // Skip newlines at the top level
            if self.check(&TokenType::Newline) {
                self.advance();
                continue;
            }

            tables.push(self.table()?);
        }

        if tables.is_empty() {
            let diagnostic = self
                .diagnostic_collector
                .parse_error(0, "TBL file must contain at least one table".to_string())
                .with_suggestion("Add a table declaration like '#my_table'".to_string());

            return Err(ParseError::UnexpectedEof {
                expected: "table declaration".to_string(),
                diagnostic: Box::new(diagnostic),
            });
        }

        Ok(Program::new(tables))
    }

    /// Parses a table: #id[flags] followed by rules
    fn table(&mut self) -> ParseResult<Node<Table>> {
        let start_pos = self.peek().span.start;

        // Expect hash symbol
        self.consume(&TokenType::Hash, "Expected '#' to start table declaration")?;

        // Expect table identifier
        let table_id = if let TokenType::Identifier(name) = &self.advance().token_type {
            name.clone()
        } else {
            let token = self.previous();
            let diagnostic = self
                .diagnostic_collector
                .parse_error(
                    token.span.start,
                    format!(
                        "Expected table identifier after '#', but found {}",
                        token.token_type
                    ),
                )
                .with_suggestion(
                    "Table names should be identifiers like 'shape' or 'my_table'".to_string(),
                );

            return Err(ParseError::UnexpectedToken {
                expected: "table identifier".to_string(),
                found: format!("{}", token.token_type),
                diagnostic: Box::new(diagnostic),
            });
        };

        // Create metadata with default values
        let mut metadata = TableMetadata::new(table_id);

        // Check for optional flags
        if self.check(&TokenType::LeftBracket) {
            self.advance(); // consume '['

            // Parse flags
            while !self.check(&TokenType::RightBracket) && !self.is_at_end() {
                if self.check(&TokenType::Export) {
                    self.advance();
                    metadata = metadata.with_export(true);
                } else {
                    let token = self.peek();
                    let diagnostic = self
                        .diagnostic_collector
                        .parse_error(
                            token.span.start,
                            format!("Unknown flag '{}' in table declaration", token.token_type),
                        )
                        .with_suggestion("Valid flags are: export".to_string());

                    return Err(ParseError::UnexpectedToken {
                        expected: "export flag or ']'".to_string(),
                        found: format!("{}", token.token_type),
                        diagnostic: Box::new(diagnostic),
                    });
                }
            }

            self.consume(&TokenType::RightBracket, "Expected ']' after table flags")?;
        }

        // Skip optional newlines after table declaration
        while self.check(&TokenType::Newline) {
            self.advance();
        }

        // Parse rules for this table
        let mut rules = Vec::new();
        while !self.is_at_end() && !self.check(&TokenType::Hash) {
            // Skip newlines between rules
            if self.check(&TokenType::Newline) {
                self.advance();
                continue;
            }

            rules.push(self.rule()?);
        }

        let end_pos = if let Some(last_rule) = rules.last() {
            last_rule.span.end
        } else {
            self.previous().span.end
        };

        let table = Table::new(metadata, rules);
        Ok(Node::new(table, Span::new(start_pos, end_pos)))
    }

    /// Parses a single rule: weight: rule_text
    fn rule(&mut self) -> ParseResult<Node<Rule>> {
        let start_pos = self.peek().span.start;

        // Expect a number (weight)
        let weight = if let TokenType::Number(n) = &self.advance().token_type {
            *n
        } else {
            let token = self.previous();
            let suggestion = match &token.token_type {
                TokenType::RuleText(_) => Some("Rules must start with a weight. Try adding a number like '1.0:' before the rule text".to_string()),
                TokenType::Colon => Some("Missing weight before colon. Try adding a number like '1.0' before the ':'".to_string()),
                TokenType::Eof => Some("File ended unexpectedly. Add a weight and rule like '1.0: some rule'".to_string()),
                _ => Some("Expected a positive number (weight) at the start of each rule".to_string()),
            };

            let diagnostic = self
                .diagnostic_collector
                .parse_error(
                    token.span.start,
                    format!(
                        "Expected positive number (weight), but found {}",
                        token.token_type
                    ),
                )
                .with_suggestion(suggestion.unwrap());

            return Err(ParseError::UnexpectedToken {
                expected: "positive number (weight)".to_string(),
                found: format!("{}", token.token_type),
                diagnostic: Box::new(diagnostic),
            });
        };

        // Expect a colon
        self.consume(&TokenType::Colon, "Expected ':' after weight")?;

        // Expect rule text
        let text = if let TokenType::RuleText(text) = &self.advance().token_type {
            text.clone()
        } else {
            let token = self.previous();
            let suggestion = match &token.token_type {
                TokenType::Newline => Some("Missing rule text after colon. Add some text describing the rule".to_string()),
                TokenType::Eof => Some("File ended after colon. Add rule text like 'some rule description'".to_string()),
                TokenType::Number(_) => Some("Found another number when expecting rule text. Each rule should have format 'weight: text'".to_string()),
                _ => Some("Expected rule text after the colon".to_string()),
            };

            let diagnostic = self
                .diagnostic_collector
                .parse_error(
                    token.span.start,
                    format!(
                        "Expected rule text after colon, but found {}",
                        token.token_type
                    ),
                )
                .with_suggestion(suggestion.unwrap());

            return Err(ParseError::UnexpectedToken {
                expected: "rule text after colon".to_string(),
                found: format!("{}", token.token_type),
                diagnostic: Box::new(diagnostic),
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

            let diagnostic = self
                .diagnostic_collector
                .parse_error(token.span.start, message.to_string())
                .with_suggestion(suggestion.unwrap());

            Err(ParseError::UnexpectedToken {
                expected: message.to_string(),
                found: format!("{}", token.token_type),
                diagnostic: Box::new(diagnostic),
            })
        }
    }
}
