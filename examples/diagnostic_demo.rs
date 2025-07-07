use parser::{parse, DiagnosticFormatter, DiagnosticCollector};

fn main() {
    println!("🔧 Advanced Diagnostic Usage Example");
    println!("====================================");
    println!("Demonstrating advanced diagnostic features with TBL (Table) language");
    println!("TBL syntax: #table_id[flags] followed by weight: rule entries\n");

    // Example 1: Basic parsing with default error formatting
    println!("📝 Example 1: Basic parsing (missing table declaration)");
    let source1 = "invalid-weight: some rule";
    match parse(source1) {
        Ok(ast) => println!("✅ Parsed: {:?}", ast),
        Err(e) => println!("❌ Error:\n{}", e),
    }

    // Example 2: Custom diagnostic formatting
    println!("\n📝 Example 2: Custom formatting (no colors, no suggestions)");
    let source2 = "#table\n2.5 missing colon";
    match parse(source2) {
        Ok(ast) => println!("✅ Parsed: {:?}", ast),
        Err(e) => {
            // Extract the diagnostic from the error
            let diagnostic = match &e {
                parser::ParseError::UnexpectedToken { diagnostic, .. } => diagnostic.as_ref(),
                parser::ParseError::UnexpectedEof { diagnostic, .. } => diagnostic.as_ref(),
                parser::ParseError::InvalidCharacter { diagnostic, .. } => diagnostic.as_ref(),
                parser::ParseError::InvalidNumber { diagnostic, .. } => diagnostic.as_ref(),
            };
            
            // Use custom formatter
            let formatter = DiagnosticFormatter::new()
                .with_colors(false)
                .with_suggestions(false);
            
            println!("❌ Custom formatted error:\n{}", formatter.format(diagnostic));
        }
    }

    // Example 3: Creating diagnostics manually
    println!("\n📝 Example 3: Manual diagnostic creation");
    let source3 = "#example\n1.0: first rule\n2.0: second rule";
    let collector = DiagnosticCollector::new(source3.to_string());
    
    // Create a custom diagnostic
    let custom_diagnostic = collector.parse_error(30, "This is a custom error message".to_string())
        .with_suggestion("Try doing something different".to_string());
    
    println!("🔧 Custom diagnostic:");
    println!("{}", custom_diagnostic);

    // Example 4: Multiple diagnostics formatting
    println!("\n📝 Example 4: Multiple diagnostics");
    let diagnostics = vec![
        collector.lex_error(0, "First error".to_string())
            .with_suggestion("Fix the first issue".to_string()),
        collector.parse_error(10, "Second error".to_string())
            .with_suggestion("Fix the second issue".to_string()),
    ];
    
    let formatter = DiagnosticFormatter::new();
    println!("🔧 Multiple diagnostics:\n{}", formatter.format_multiple(&diagnostics));

    println!("\n🎯 Key Benefits of the New Architecture:");
    println!("   • Clean separation between error collection and formatting");
    println!("   • Structured diagnostic data that can be processed programmatically");
    println!("   • Customizable error formatting and presentation");
    println!("   • Rich contextual information with line/column positions");
    println!("   • Helpful suggestions for fixing errors");
}
