use crate::ast::{BinaryOp, Expr, Literal, Node, Program, Span, Stmt, UnaryOp};
use crate::errors::{ParseError, ParseResult};
use crate::lexer::{Token, TokenType};

/// Recursive descent parser for our language
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
        let mut statements = Vec::new();

        while !self.is_at_end() {
            // Skip newlines at the top level
            if self.check(&TokenType::Newline) {
                self.advance();
                continue;
            }

            statements.push(self.statement()?);
        }

        Ok(Program::new(statements))
    }

    /// Parses a statement
    fn statement(&mut self) -> ParseResult<Node<Stmt>> {
        let start_pos = self.peek().span.start;

        let stmt = if self.match_token(&TokenType::Let) {
            self.var_declaration()?
        } else if self.match_token(&TokenType::If) {
            self.if_statement()?
        } else if self.match_token(&TokenType::While) {
            self.while_statement()?
        } else if self.match_token(&TokenType::Return) {
            self.return_statement()?
        } else if self.match_token(&TokenType::LeftBrace) {
            Stmt::Block(self.block()?)
        } else {
            self.expression_statement()?
        };

        let end_pos = self.previous().span.end;
        Ok(Node::new(stmt, Span::new(start_pos, end_pos)))
    }

    fn var_declaration(&mut self) -> ParseResult<Stmt> {
        let name = if let TokenType::Identifier(name) = &self
            .consume(
                &TokenType::Identifier("".to_string()),
                "Expected variable name",
            )?
            .token_type
        {
            name.clone()
        } else {
            return Err(ParseError::SyntaxError {
                message: "Expected variable name".to_string(),
                position: self.peek().span.start,
            });
        };

        let initializer = if self.match_token(&TokenType::Equal) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            &TokenType::Semicolon,
            "Expected ';' after variable declaration",
        )?;

        Ok(Stmt::VarDecl { name, initializer })
    }

    fn if_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(&TokenType::LeftParen, "Expected '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RightParen, "Expected ')' after if condition")?;

        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.match_token(&TokenType::Else) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(&TokenType::LeftParen, "Expected '(' after 'while'")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RightParen, "Expected ')' after while condition")?;

        let body = Box::new(self.statement()?);

        Ok(Stmt::While { condition, body })
    }

    fn return_statement(&mut self) -> ParseResult<Stmt> {
        let value = if self.check(&TokenType::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };

        self.consume(&TokenType::Semicolon, "Expected ';' after return value")?;
        Ok(Stmt::Return(value))
    }

    fn block(&mut self) -> ParseResult<Vec<Node<Stmt>>> {
        let mut statements = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            // Skip newlines in blocks
            if self.check(&TokenType::Newline) {
                self.advance();
                continue;
            }

            statements.push(self.statement()?);
        }

        self.consume(&TokenType::RightBrace, "Expected '}' after block")?;
        Ok(statements)
    }

    fn expression_statement(&mut self) -> ParseResult<Stmt> {
        // Check for assignment: identifier = expression
        if self.check_assignment() {
            let name = if let TokenType::Identifier(name) = &self.advance().token_type {
                name.clone()
            } else {
                unreachable!() // We already checked this is an identifier
            };

            self.consume(&TokenType::Equal, "Expected '=' in assignment")?;
            let value = self.expression()?;
            self.consume(&TokenType::Semicolon, "Expected ';' after assignment")?;

            Ok(Stmt::Assignment { name, value })
        } else {
            let expr = self.expression()?;
            self.consume(&TokenType::Semicolon, "Expected ';' after expression")?;
            Ok(Stmt::Expression(expr))
        }
    }

    /// Parses an expression using operator precedence
    fn expression(&mut self) -> ParseResult<Node<Expr>> {
        self.or()
    }

    fn or(&mut self) -> ParseResult<Node<Expr>> {
        let mut expr = self.and()?;

        while self.match_token(&TokenType::OrOr) {
            let operator = BinaryOp::Or;
            let right = self.and()?;
            let span = Span::new(expr.span.start, right.span.end);
            expr = Node::new(
                Expr::Binary {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(expr)
    }

    fn and(&mut self) -> ParseResult<Node<Expr>> {
        let mut expr = self.equality()?;

        while self.match_token(&TokenType::AndAnd) {
            let operator = BinaryOp::And;
            let right = self.equality()?;
            let span = Span::new(expr.span.start, right.span.end);
            expr = Node::new(
                Expr::Binary {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(expr)
    }

    fn equality(&mut self) -> ParseResult<Node<Expr>> {
        let mut expr = self.comparison()?;

        while let Some(operator) = self.match_equality_op() {
            let right = self.comparison()?;
            let span = Span::new(expr.span.start, right.span.end);
            expr = Node::new(
                Expr::Binary {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> ParseResult<Node<Expr>> {
        let mut expr = self.term()?;

        while let Some(operator) = self.match_comparison_op() {
            let right = self.term()?;
            let span = Span::new(expr.span.start, right.span.end);
            expr = Node::new(
                Expr::Binary {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(expr)
    }

    fn term(&mut self) -> ParseResult<Node<Expr>> {
        let mut expr = self.factor()?;

        while let Some(operator) = self.match_term_op() {
            let right = self.factor()?;
            let span = Span::new(expr.span.start, right.span.end);
            expr = Node::new(
                Expr::Binary {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(expr)
    }

    fn factor(&mut self) -> ParseResult<Node<Expr>> {
        let mut expr = self.unary()?;

        while let Some(operator) = self.match_factor_op() {
            let right = self.unary()?;
            let span = Span::new(expr.span.start, right.span.end);
            expr = Node::new(
                Expr::Binary {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                },
                span,
            );
        }

        Ok(expr)
    }

    fn unary(&mut self) -> ParseResult<Node<Expr>> {
        if let Some(operator) = self.match_unary_op() {
            let start_pos = self.previous().span.start;
            let right = self.unary()?;
            let span = Span::new(start_pos, right.span.end);
            Ok(Node::new(
                Expr::Unary {
                    operator,
                    operand: Box::new(right),
                },
                span,
            ))
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> ParseResult<Node<Expr>> {
        let mut expr = self.primary()?;

        while self.match_token(&TokenType::LeftParen) {
            expr = self.finish_call(expr)?;
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Node<Expr>) -> ParseResult<Node<Expr>> {
        let mut arguments = Vec::new();

        if !self.check(&TokenType::RightParen) {
            loop {
                arguments.push(self.expression()?);
                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
        }

        let closing_paren = self.consume(&TokenType::RightParen, "Expected ')' after arguments")?;
        let span = Span::new(callee.span.start, closing_paren.span.end);

        Ok(Node::new(
            Expr::Call {
                function: Box::new(callee),
                arguments,
            },
            span,
        ))
    }

    fn primary(&mut self) -> ParseResult<Node<Expr>> {
        let token = self.advance();
        let span = token.span;

        match &token.token_type {
            TokenType::True => Ok(Node::new(Expr::Literal(Literal::Boolean(true)), span)),
            TokenType::False => Ok(Node::new(Expr::Literal(Literal::Boolean(false)), span)),
            TokenType::Null => Ok(Node::new(Expr::Literal(Literal::Null), span)),
            TokenType::Number(n) => Ok(Node::new(Expr::Literal(Literal::Number(*n)), span)),
            TokenType::String(s) => Ok(Node::new(Expr::Literal(Literal::String(s.clone())), span)),
            TokenType::Identifier(name) => Ok(Node::new(Expr::Identifier(name.clone()), span)),
            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.consume(&TokenType::RightParen, "Expected ')' after expression")?;
                let end_span = self.previous().span.end;
                let group_span = Span::new(span.start, end_span);
                Ok(Node::new(Expr::Group(Box::new(expr)), group_span))
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "expression".to_string(),
                found: format!("{}", token.token_type),
                position: token.span.start,
            }),
        }
    }

    // Helper methods for matching operators
    fn match_equality_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(&TokenType::BangEqual) {
            Some(BinaryOp::NotEqual)
        } else if self.match_token(&TokenType::EqualEqual) {
            Some(BinaryOp::Equal)
        } else {
            None
        }
    }

    fn match_comparison_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(&TokenType::Greater) {
            Some(BinaryOp::Greater)
        } else if self.match_token(&TokenType::GreaterEqual) {
            Some(BinaryOp::GreaterEqual)
        } else if self.match_token(&TokenType::Less) {
            Some(BinaryOp::Less)
        } else if self.match_token(&TokenType::LessEqual) {
            Some(BinaryOp::LessEqual)
        } else {
            None
        }
    }

    fn match_term_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(&TokenType::Minus) {
            Some(BinaryOp::Subtract)
        } else if self.match_token(&TokenType::Plus) {
            Some(BinaryOp::Add)
        } else {
            None
        }
    }

    fn match_factor_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(&TokenType::Slash) {
            Some(BinaryOp::Divide)
        } else if self.match_token(&TokenType::Star) {
            Some(BinaryOp::Multiply)
        } else if self.match_token(&TokenType::Percent) {
            Some(BinaryOp::Modulo)
        } else {
            None
        }
    }

    fn match_unary_op(&mut self) -> Option<UnaryOp> {
        if self.match_token(&TokenType::Bang) {
            Some(UnaryOp::Not)
        } else if self.match_token(&TokenType::Minus) {
            Some(UnaryOp::Minus)
        } else {
            None
        }
    }

    // Helper method to check if we're looking at an assignment
    fn check_assignment(&self) -> bool {
        if matches!(self.peek().token_type, TokenType::Identifier(_)) {
            // Look ahead to see if the next token is '='
            if self.current + 1 < self.tokens.len() {
                matches!(self.tokens[self.current + 1].token_type, TokenType::Equal)
            } else {
                false
            }
        } else {
            false
        }
    }

    // Utility methods
    fn match_token(&mut self, token_type: &TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

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
