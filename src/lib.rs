pub mod ast;
pub mod diagnostic;
pub mod diagnostic_collector;
pub mod diagnostic_formatter;
pub mod errors;
pub mod lexer;
pub mod parser;

pub use ast::{Node, Program, Rule, Span, Table, TableMetadata};
pub use diagnostic::{Diagnostic, DiagnosticKind, Severity, SourceLocation};
pub use diagnostic_collector::DiagnosticCollector;
pub use diagnostic_formatter::DiagnosticFormatter;
pub use errors::{LexError, LexResult, ParseError, ParseResult};
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
    fn test_simple_table() {
        let source = "#shape\n1.5: simple rule";
        let result = parse(source);
        assert!(result.is_ok());
        let program = result.unwrap();
        assert_eq!(program.tables.len(), 1);
        assert_eq!(program.tables[0].value.metadata.id, "shape");
        assert_eq!(program.tables[0].value.metadata.export, false);
        assert_eq!(program.tables[0].value.rules.len(), 1);
        assert_eq!(program.tables[0].value.rules[0].value.weight, 1.5);
        assert_eq!(program.tables[0].value.rules[0].value.text, "simple rule");
    }

    #[test]
    fn test_table_with_export_flag() {
        let source = "#shape[export]\n1.0: circle\n2.0: square";
        let result = parse(source);
        assert!(result.is_ok());
        let program = result.unwrap();
        assert_eq!(program.tables.len(), 1);
        assert_eq!(program.tables[0].value.metadata.id, "shape");
        assert_eq!(program.tables[0].value.metadata.export, true);
        assert_eq!(program.tables[0].value.rules.len(), 2);
    }

    #[test]
    fn test_multiple_tables() {
        let source = r#"#shapes
1.0: circle
2.5: square

#colors[export]
1.0: red
3.0: blue"#;
        let result = parse(source);
        assert!(result.is_ok());
        let program = result.unwrap();
        assert_eq!(program.tables.len(), 2);

        // First table
        assert_eq!(program.tables[0].value.metadata.id, "shapes");
        assert_eq!(program.tables[0].value.metadata.export, false);
        assert_eq!(program.tables[0].value.rules.len(), 2);

        // Second table
        assert_eq!(program.tables[1].value.metadata.id, "colors");
        assert_eq!(program.tables[1].value.metadata.export, true);
        assert_eq!(program.tables[1].value.rules.len(), 2);
    }

    #[test]
    fn test_tokenize() {
        let source = "#test\n1.5: test rule";
        let result = tokenize(source);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(!tokens.is_empty());
        assert!(matches!(tokens.last().unwrap().token_type, TokenType::Eof));
    }

    #[test]
    fn test_invalid_negative_weight() {
        let source = "#test\n-1.0: invalid rule";
        let result = parse(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_colon() {
        let source = "#test\n1.5 missing colon";
        let result = parse(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_input() {
        let source = "";
        let result = parse(source);
        assert!(result.is_err()); // TBL requires at least one table
    }

    #[test]
    fn test_table_with_spaces() {
        let source = "#test\n3.14: rule with multiple   spaces";
        let result = parse(source);
        assert!(result.is_ok());
        let program = result.unwrap();
        assert_eq!(program.tables.len(), 1);
        assert_eq!(program.tables[0].value.rules.len(), 1);
        assert_eq!(
            program.tables[0].value.rules[0].value.text,
            "rule with multiple   spaces"
        );
    }
}
