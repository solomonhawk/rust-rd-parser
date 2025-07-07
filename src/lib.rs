pub mod ast;
pub mod errors;
pub mod lexer;
pub mod parser;

pub use ast::{Expr, Literal, Node, Program, Span, Stmt};
pub use errors::{LexError, ParseError, ParseResult};
pub use lexer::{Lexer, Token, TokenType};
pub use parser::Parser;

/// Parse source code into an AST
///
/// This is the main entry point for parsing. It takes source code as a string
/// and returns either a parsed AST or an error.
///
/// # Examples
///
/// ```
/// use parser::parse;
///
/// let source = "let x = 42;";
/// match parse(source) {
///     Ok(ast) => println!("Parsed successfully: {:?}", ast),
///     Err(e) => eprintln!("Parse error: {}", e),
/// }
/// ```
pub fn parse(source: &str) -> ParseResult<Program> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

/// Tokenize source code into tokens
///
/// This function takes source code and returns a vector of tokens or an error.
/// This is useful if you want to inspect the tokens before parsing.
///
/// # Examples
///
/// ```
/// use parser::tokenize;
///
/// let source = "let x = 42;";
/// match tokenize(source) {
///     Ok(tokens) => {
///         for token in tokens {
///             println!("{:?}", token);
///         }
///     }
///     Err(e) => eprintln!("Lexer error: {}", e),
/// }
/// ```
pub fn tokenize(source: &str) -> Result<Vec<Token>, LexError> {
    let mut lexer = Lexer::new(source);
    lexer.tokenize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_expression() {
        let source = "42 + 3;";
        let result = parse(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_variable_declaration() {
        let source = "let x = 42;";
        let result = parse(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_tokenize() {
        let source = "let x = 42;";
        let result = tokenize(source);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(!tokens.is_empty());
        assert!(matches!(tokens.last().unwrap().token_type, TokenType::Eof));
    }

    #[test]
    fn test_invalid_syntax() {
        let source = "let x = ;"; // Missing value
        let result = parse(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_complex_expression() {
        let source = "let result = (42 + 3) * 2 - 1;";
        let result = parse(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_if_statement() {
        let source = r#"
            if (x > 0) {
                return x;
            } else {
                return -x;
            }
        "#;
        let result = parse(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_while_loop() {
        let source = r#"
            while (i < 10) {
                i = i + 1;
            }
        "#;
        let result = parse(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_call() {
        let source = "print(\"Hello, world!\");";
        let result = parse(source);
        assert!(result.is_ok());
    }
}
