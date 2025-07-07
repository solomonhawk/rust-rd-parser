pub mod ast;
pub mod errors;
pub mod lexer;
pub mod parser;

pub use ast::{Node, Program, Rule, Span};
pub use errors::{LexError, ParseError, ParseResult, LexResult, ErrorContext};
pub use lexer::{Lexer, Token, TokenType};
pub use parser::Parser;

/// Parse source code into an AST
///
/// This is the main entry point for parsing. It takes source code as a string
/// and returns either a parsed AST or an error with enhanced error reporting.
///
/// # Examples
///
/// ```
/// use parser::parse;
///
/// let source = "1.5: simple rule";
/// match parse(source) {
///     Ok(ast) => println!("Parsed successfully: {:?}", ast),
///     Err(e) => eprintln!("Parse error:\n{}", e),
/// }
/// ```
pub fn parse(source: &str) -> ParseResult<Program> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::from_source(tokens, source.to_string());
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
/// let source = "1.5: test rule";
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
    fn test_simple_rule() {
        let source = "1.5: simple rule";
        let result = parse(source);
        assert!(result.is_ok());
        let program = result.unwrap();
        assert_eq!(program.rules.len(), 1);
        assert_eq!(program.rules[0].value.weight, 1.5);
        assert_eq!(program.rules[0].value.text, "simple rule");
    }

    #[test]
    fn test_multiple_rules() {
        let source = r#"1.0: first rule
2.5: second rule
10.0: third rule"#;
        let result = parse(source);
        assert!(result.is_ok());
        let program = result.unwrap();
        assert_eq!(program.rules.len(), 3);
        assert_eq!(program.rules[0].value.weight, 1.0);
        assert_eq!(program.rules[1].value.weight, 2.5);
        assert_eq!(program.rules[2].value.weight, 10.0);
    }

    #[test]
    fn test_tokenize() {
        let source = "1.5: test rule";
        let result = tokenize(source);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(!tokens.is_empty());
        assert!(matches!(tokens.last().unwrap().token_type, TokenType::Eof));
    }

    #[test]
    fn test_invalid_negative_weight() {
        let source = "-1.0: invalid rule";
        let result = parse(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_colon() {
        let source = "1.5 missing colon";
        let result = parse(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_input() {
        let source = "";
        let result = parse(source);
        assert!(result.is_ok());
        let program = result.unwrap();
        assert_eq!(program.rules.len(), 0);
    }

    #[test]
    fn test_rule_with_spaces() {
        let source = "3.14: rule with multiple   spaces";
        let result = parse(source);
        assert!(result.is_ok());
        let program = result.unwrap();
        assert_eq!(program.rules.len(), 1);
        assert_eq!(program.rules[0].value.text, "rule with multiple   spaces");
    }
}
