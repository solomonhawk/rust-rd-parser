use table_collection::parse;

pub fn main() {
    println!("🚀 Rust TBL (Table) Language Parser Demo");
    println!("=========================================");
    println!("This parser handles TBL format: #table_id[flags] followed by weight: rule entries");
    println!("Let's test various inputs and see the enhanced error reporting!\n");

    // Test successful cases first
    let success_examples = vec![
        ("#shape\n1.5: simple rule", "Basic table"),
        (
            "#shapes\n1.0: circle\n2.5: square\n10.0: triangle",
            "Single table with multiple rules",
        ),
        (
            "#shapes[export]\n3.14: circle\n0.5: square",
            "Table with export flag",
        ),
        (
            r#"#shapes
1.0: circle
2.5: square

#colors[export]
1.0: red
3.0: blue"#,
            "Multiple tables",
        ),
        ("#test\n0.001: very small weight", "Small weight"),
        ("#test\n999.999: very large weight", "Large weight"),
        ("#test\n1.0: {#color}", "Simple table reference expression"),
        ("#item\n1: {#color} {#shape}", "Multiple table references"),
        (
            "#test\n1.0: prefix {#table} suffix",
            "Mixed content with expression",
        ),
    ];

    println!("✅ SUCCESSFUL PARSING EXAMPLES");
    println!("==============================\n");

    for (example, description) in success_examples {
        println!("📝 {}", description);
        println!("Source: {}", example.replace('\n', "\\n"));

        match parse(example) {
            Ok(program) => {
                let total_rules: usize = program.tables.iter().map(|t| t.value.rules.len()).sum();
                println!(
                    "✅ Parsed {} table(s) with {} total rules!",
                    program.tables.len(),
                    total_rules
                );
                for (i, table) in program.tables.iter().enumerate() {
                    println!(
                        "   Table {}: id='{}', export={}, {} rules",
                        i + 1,
                        table.value.metadata.id,
                        table.value.metadata.export,
                        table.value.rules.len()
                    );
                    for (j, rule) in table.value.rules.iter().enumerate() {
                        println!(
                            "     Rule {}: weight={}, text=\"{}\"",
                            j + 1,
                            rule.value.weight,
                            rule.value.to_string()
                        );
                    }
                }
            }
            Err(e) => {
                println!("❌ Unexpected error: {}", e);
            }
        }
        println!();
    }

    // Now test error cases with enhanced reporting
    let error_examples = vec![
        (
            "#test\n-1.0: negative weight",
            "Negative weight (should be positive)",
        ),
        ("#test\n0: zero weight", "Zero weight (should be positive)"),
        ("abc: not a number", "Missing table declaration"),
        (
            "#test\n1.5 missing colon after weight",
            "Missing colon separator",
        ),
        ("#test\n1.5:", "Missing rule text after colon"),
        ("#\n1.5: missing table name", "Missing table identifier"),
        ("#test[unknown]\n1.5: unknown flag", "Unknown table flag"),
        (
            "#test[export\n1.5: missing bracket",
            "Missing closing bracket",
        ),
        ("", "Empty input"),
        ("   \n  \n", "Only whitespace"),
        ("#test\n1.5.5.5: too many dots", "Invalid number format"),
        (
            "#test\n1e5: scientific notation",
            "Scientific notation (not supported)",
        ),
    ];

    println!("❌ ERROR HANDLING EXAMPLES");
    println!("==========================");
    println!(
        "These examples show off our enhanced error reporting with context and suggestions!\n"
    );

    for (example, description) in error_examples {
        println!("🔍 {}", description);
        println!("Source: \"{}\"", example.replace('\n', "\\n"));

        match parse(example) {
            Ok(program) => {
                let total_rules: usize = program.tables.iter().map(|t| t.value.rules.len()).sum();
                println!(
                    "✅ Unexpectedly parsed {} table(s) with {} total rules:",
                    program.tables.len(),
                    total_rules
                );
                for (i, table) in program.tables.iter().enumerate() {
                    println!(
                        "   Table {}: id='{}', export={}, {} rules",
                        i + 1,
                        table.value.metadata.id,
                        table.value.metadata.export,
                        table.value.rules.len()
                    );
                }
            }
            Err(e) => {
                println!("{}", e);
            }
        }
        println!("{}", "─".repeat(50));
    }

    println!("\n🎉 TBL Parser Demo complete! Notice how the error messages provide:");
    println!("   • Exact line and column positions");
    println!("   • Visual pointers to the problem location");
    println!("   • Context-aware suggestions for fixing the issue");
    println!("   • Clear explanations of what went wrong");
    println!("   • Full support for table-based language structure");
}
