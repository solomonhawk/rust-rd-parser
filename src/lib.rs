pub mod ast;
pub mod collection;
pub mod diagnostic;
pub mod diagnostic_collector;
pub mod diagnostic_formatter;
pub mod errors;
pub mod lexer;
pub mod parser;

#[cfg(feature = "wasm")]
pub mod wasm;

pub use ast::{Expression, Node, Program, Rule, RuleContent, Span, Table, TableMetadata};
pub use collection::{Collection, CollectionError, CollectionGenResult, CollectionResult};
pub use diagnostic::{Diagnostic, DiagnosticKind, Severity, SourceLocation};
pub use diagnostic_collector::DiagnosticCollector;
pub use diagnostic_formatter::DiagnosticFormatter;
pub use errors::{LexError, LexResult, ParseError, ParseResult};
pub use lexer::{Lexer, Token, TokenType};

#[cfg(feature = "wasm")]
pub use wasm::{WasmCollection, WasmParser, WasmUtils};

use crate::parser::Parser;

/// Parse source code into an AST
///
/// This is the main entry point for parsing. It takes source code as a string
/// and returns either a parsed AST or an error with enhanced error reporting.
///
/// # Examples
///
/// ```
/// use table_collection::parse;
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
/// use table_collection::tokenize;
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
        assert!(!program.tables[0].value.metadata.export);
        assert_eq!(program.tables[0].value.rules.len(), 1);
        assert_eq!(program.tables[0].value.rules[0].value.weight, 1.5);
        assert_eq!(
            program.tables[0].value.rules[0].value.content_text(),
            "simple rule"
        );
    }

    #[test]
    fn test_table_with_export_flag() {
        let source = "#shape[export]\n1.0: circle\n2.0: square";
        let result = parse(source);
        assert!(result.is_ok());
        let program = result.unwrap();
        assert_eq!(program.tables.len(), 1);
        assert_eq!(program.tables[0].value.metadata.id, "shape");
        assert!(program.tables[0].value.metadata.export);
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
        assert!(!program.tables[0].value.metadata.export);
        assert_eq!(program.tables[0].value.rules.len(), 2);

        // Second table
        assert_eq!(program.tables[1].value.metadata.id, "colors");
        assert!(program.tables[1].value.metadata.export);
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
            program.tables[0].value.rules[0].value.content_text(),
            "rule with multiple   spaces"
        );
    }

    #[test]
    fn test_table_reference_expressions() {
        let source = r#"#color
1.0: red
2.0: blue
3.0: green

#shape
1.0: circle
2.0: square

#item
1.0: {#color} {#shape}
2.0: big {#color} {#shape}
3.0: small {#shape}"#;

        let result = parse(source);
        assert!(result.is_ok());
        let program = result.unwrap();
        assert_eq!(program.tables.len(), 3);

        // Color table
        assert_eq!(program.tables[0].value.metadata.id, "color");
        assert_eq!(program.tables[0].value.rules.len(), 3);

        // Shape table
        assert_eq!(program.tables[1].value.metadata.id, "shape");
        assert_eq!(program.tables[1].value.rules.len(), 2);

        // Item table with expressions
        assert_eq!(program.tables[2].value.metadata.id, "item");
        assert_eq!(program.tables[2].value.rules.len(), 3);

        // Check the first rule: "1.0: {#color} {#shape}"
        let rule1 = &program.tables[2].value.rules[0].value;
        assert_eq!(rule1.weight, 1.0);
        assert_eq!(rule1.content.len(), 4); // space, {#color}, space, {#shape}
        match &rule1.content[0] {
            RuleContent::Text(text) => assert_eq!(text, " "),
            _ => panic!("Expected text content"),
        }
        match &rule1.content[1] {
            RuleContent::Expression(Expression::TableReference { table_id }) => {
                assert_eq!(table_id, "color");
            }
            _ => panic!("Expected table reference expression"),
        }
        match &rule1.content[2] {
            RuleContent::Text(text) => assert_eq!(text, " "),
            _ => panic!("Expected text content"),
        }
        match &rule1.content[3] {
            RuleContent::Expression(Expression::TableReference { table_id }) => {
                assert_eq!(table_id, "shape");
            }
            _ => panic!("Expected table reference expression"),
        }

        // Check content_text() works correctly for expressions
        assert_eq!(rule1.content_text(), "{#color} {#shape}");

        // Check the second rule: "2.0: big {#color} {#shape}"
        let rule2 = &program.tables[2].value.rules[1].value;
        assert_eq!(rule2.content_text(), "big {#color} {#shape}");

        // Check the third rule: "3.0: small {#shape}"
        let rule3 = &program.tables[2].value.rules[2].value;
        assert_eq!(rule3.content_text(), "small {#shape}");
    }

    #[test]
    fn test_table_ids_with_hyphens_and_underscores() {
        let source = r#"#potion-descriptor
1.0: fresh
2.0: stale

#color_variant
1.0: light blue
2.0: dark red

#item-with_mixed
1.0: {#potion-descriptor} {#color_variant} potion"#;

        let result = parse(source);
        assert!(
            result.is_ok(),
            "Should parse table IDs with hyphens and underscores"
        );

        let program = result.unwrap();
        assert_eq!(program.tables.len(), 3);

        // Check that all table IDs are parsed correctly
        let table_ids: Vec<&str> = program
            .tables
            .iter()
            .map(|table| table.value.metadata.id.as_str())
            .collect();

        assert!(table_ids.contains(&"potion-descriptor"));
        assert!(table_ids.contains(&"color_variant"));
        assert!(table_ids.contains(&"item-with_mixed"));
    }

    #[test]
    fn test_collection_with_hyphenated_table_ids() {
        let source = r#"#potion-descriptor
1.0: fresh
2.0: stale

#color-variant
1.0: light blue
2.0: dark red

#mixed-item
1.0: {#potion-descriptor} {#color-variant} potion"#;

        let result = Collection::new(source);
        assert!(
            result.is_ok(),
            "Collection should work with hyphenated table IDs"
        );

        let mut collection = result.unwrap();

        // Test that we can generate from tables with hyphens
        assert!(collection.has_table("potion-descriptor"));
        assert!(collection.has_table("color-variant"));
        assert!(collection.has_table("mixed-item"));

        let generation_result = collection.generate("mixed-item", 1);
        assert!(
            generation_result.is_ok(),
            "Should be able to generate from hyphenated table"
        );
    }

    #[test]
    fn test_tokenize_hyphenated_identifiers() {
        let source = "#potion-descriptor";
        let result = tokenize(source);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 3); // Hash, Identifier, EOF

        if let TokenType::Identifier(name) = &tokens[1].token_type {
            assert_eq!(name, "potion-descriptor");
        } else {
            panic!("Expected identifier token with hyphens");
        }
    }

    #[test]
    fn test_line_comments() {
        let source = r#"// This is a line comment
#test
1.0: rule text // comment at end of line
2.0: another rule"#;

        let result = parse(source);
        assert!(result.is_ok(), "Should parse with line comments");

        let program = result.unwrap();
        assert_eq!(program.tables.len(), 1);
        assert_eq!(program.tables[0].value.rules.len(), 2);
    }

    #[test]
    fn test_block_comments() {
        let source = r#"/* This is a 
           multi-line 
           block comment */
#test
1.0: rule /* inline comment */ text
2.0: another rule"#;

        let result = parse(source);
        assert!(result.is_ok(), "Should parse with block comments");

        let program = result.unwrap();
        assert_eq!(program.tables.len(), 1);
        assert_eq!(program.tables[0].value.rules.len(), 2);
    }

    #[test]
    fn test_comments_in_collection() {
        let source = r#"// Comment before table
#color
1.0: red // end of line comment
/* block comment */ 2.0: blue

// Another comment
#item
1.0: {#color} item /* comment */ text"#;

        let result = Collection::new(source);
        assert!(result.is_ok(), "Collection should work with comments");

        let mut collection = result.unwrap();
        assert!(collection.has_table("color"));
        assert!(collection.has_table("item"));

        let generation_result = collection.generate("item", 1);
        assert!(
            generation_result.is_ok(),
            "Should generate with commented source"
        );
    }

    #[test]
    fn test_unterminated_block_comment() {
        let source = r#"#test
1.0: rule text /* unterminated comment
2.0: another rule"#;

        let result = parse(source);
        assert!(
            result.is_err(),
            "Should fail with unterminated block comment"
        );
    }

    #[test]
    fn test_tokenize_with_comments() {
        let source = "// comment\n#test // another\n1.0: text";
        let result = tokenize(source);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        // Should have: Hash, Identifier, Newline, Number, Colon, TextSegment, EOF
        // Comments should be filtered out
        let non_eof_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| !matches!(t.token_type, TokenType::Eof))
            .collect();

        assert!(non_eof_tokens.len() >= 5); // At least the core tokens
    }

    #[test]
    fn test_dice_roll_expressions() {
        let source = r#"#test
1.0: roll {d6}
2.0: bigger roll {2d10}
3.0: huge roll {100d20}"#;

        let result = parse(source);
        if let Err(ref e) = result {
            println!("Parse error: {}", e);
        }
        assert!(result.is_ok(), "Should parse dice roll expressions");

        let program = result.unwrap();
        assert_eq!(program.tables.len(), 1);
        assert_eq!(program.tables[0].value.rules.len(), 3);

        // Check the first rule contains a dice roll
        let rule1 = &program.tables[0].value.rules[0].value;
        assert_eq!(rule1.content.len(), 2); // "roll " and dice expression
        match &rule1.content[1] {
            RuleContent::Expression(Expression::DiceRoll { count, sides }) => {
                assert_eq!(*count, None);
                assert_eq!(*sides, 6);
            }
            _ => panic!("Expected dice roll expression"),
        }

        // Check content_text() works correctly for dice rolls
        assert_eq!(rule1.content_text(), "roll {d6}");

        let rule2 = &program.tables[0].value.rules[1].value;
        assert_eq!(rule2.content_text(), "bigger roll {2d10}");
    }

    #[test]
    fn test_dice_roll_in_collection() {
        let source = r#"#dice-test
1.0: You rolled {d6}!
2.0: Double dice: {2d6}
3.0: Big damage: {5d8} points"#;

        let result = Collection::new(source);
        assert!(result.is_ok(), "Collection should work with dice rolls");

        let mut collection = result.unwrap();
        assert!(collection.has_table("dice-test"));

        // Test generation - should produce numbers
        let generation_result = collection.generate("dice-test", 5);
        assert!(generation_result.is_ok(), "Should generate with dice rolls");

        let generated = generation_result.unwrap();
        println!("Generated with dice: {}", generated);
        // Should contain numeric results from dice rolls
        assert!(generated.contains(char::is_numeric), "Should contain dice roll results");
    }

    #[test]
    fn test_tokenize_dice_rolls() {
        let source = "#test\n1.0: {d6} {2d10} {100d20}";
        let result = tokenize(source);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        // Find the dice roll tokens
        let dice_tokens: Vec<_> = tokens.iter()
            .filter(|t| matches!(t.token_type, TokenType::DiceRoll { .. }))
            .collect();
        
        assert_eq!(dice_tokens.len(), 3, "Should have 3 dice roll tokens");
        
        // Check first dice roll (d6)
        if let TokenType::DiceRoll { count, sides } = &dice_tokens[0].token_type {
            assert_eq!(*count, None);
            assert_eq!(*sides, 6);
        } else {
            panic!("Expected dice roll token");
        }
        
        // Check second dice roll (2d10)
        if let TokenType::DiceRoll { count, sides } = &dice_tokens[1].token_type {
            assert_eq!(*count, Some(2));
            assert_eq!(*sides, 10);
        } else {
            panic!("Expected dice roll token");
        }
        
        // Check third dice roll (100d20)
        if let TokenType::DiceRoll { count, sides } = &dice_tokens[2].token_type {
            assert_eq!(*count, Some(100));
            assert_eq!(*sides, 20);
        } else {
            panic!("Expected dice roll token");
        }
    }

    #[test]
    fn test_mixed_expressions() {
        let source = r#"#mixed
1.0: {#color} sword with {d6} damage
2.0: {2d4} {#potion} bottles"#;

        let result = parse(source);
        assert!(result.is_ok(), "Should parse mixed table references and dice rolls");

        let program = result.unwrap();
        let rule1 = &program.tables[0].value.rules[0].value;

        // Should have: text, table_ref, text, dice_roll, text
        assert_eq!(rule1.content.len(), 5);
        match &rule1.content[1] {
            RuleContent::Expression(Expression::TableReference { table_id }) => {
                assert_eq!(table_id, "color");
            }
            _ => panic!("Expected table reference"),
        }
        match &rule1.content[3] {
            RuleContent::Expression(Expression::DiceRoll { count, sides }) => {
                assert_eq!(*count, None);
                assert_eq!(*sides, 6);
            }
            _ => panic!("Expected dice roll"),
        }
    }
}
