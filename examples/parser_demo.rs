use parser::parse;

pub fn main() {
    println!("🚀 Rust Recursive Descent Parser Demo");
    println!("=====================================");
    println!("This parser handles a simple format: <weight>: <rule>");
    println!("Let's test various inputs and see the enhanced error reporting!\n");

    // Test successful cases first
    let success_examples = vec![
        ("1.5: simple rule", "Basic rule"),
        (r#"1.0: first rule
2.5: second rule
10.0: third rule"#, "Multiple rules"),
        (r#"3.14: rule with numbers 123 and symbols !@#
0.5: rule with multiple   spaces
100.0: rule with punctuation, commas; and colons: but only the first colon matters
42.0: rule with "quotes" and 'apostrophes'"#, "Rules with various content"),
        ("0.001: very small weight", "Small weight"),
        ("999.999: very large weight", "Large weight"),
    ];

    println!("✅ SUCCESSFUL PARSING EXAMPLES");
    println!("==============================\n");

    for (example, description) in success_examples {
        println!("📝 {}", description);
        println!("Source: {}", example.replace('\n', "\\n"));
        
        match parse(example) {
            Ok(program) => {
                println!("✅ Parsed {} rule(s) successfully!", program.rules.len());
                for (j, rule) in program.rules.iter().enumerate() {
                    println!("   Rule {}: weight={}, text=\"{}\"", j + 1, rule.value.weight, rule.value.text);
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
        ("-1.0: negative weight", "Negative weight (should be positive)"),
        ("0: zero weight", "Zero weight (should be positive)"),
        ("abc: not a number", "Invalid weight format"),
        ("1.5 missing colon after weight", "Missing colon separator"),
        ("1.5:", "Missing rule text after colon"),
        (": missing weight before colon", "Missing weight"),
        ("1.5: valid rule\n-2.0: another negative", "Mixed valid/invalid"),
        ("1.5: valid rule\n2.0 missing colon", "Missing colon in second rule"),
        ("", "Empty input"),
        ("   \n  \n", "Only whitespace"),
        ("1.5.5.5: too many dots", "Invalid number format"),
        ("1e5: scientific notation", "Scientific notation (not supported)"),
    ];

    println!("❌ ERROR HANDLING EXAMPLES");
    println!("==========================");
    println!("These examples show off our enhanced error reporting with context and suggestions!\n");

    for (example, description) in error_examples {
        println!("🔍 {}", description);
        println!("Source: \"{}\"", example.replace('\n', "\\n"));
        
        match parse(example) {
            Ok(program) => {
                println!("✅ Unexpectedly parsed {} rule(s):", program.rules.len());
                for (j, rule) in program.rules.iter().enumerate() {
                    println!("   Rule {}: weight={}, text=\"{}\"", j + 1, rule.value.weight, rule.value.text);
                }
            }
            Err(e) => {
                println!("{}", e);
            }
        }
        println!("{}",  "─".repeat(50));
    }

    println!("\n🎉 Demo complete! Notice how the error messages provide:");
    println!("   • Exact line and column positions");
    println!("   • Visual pointers to the problem location");
    println!("   • Context-aware suggestions for fixing the issue");
    println!("   • Clear explanations of what went wrong");
}
