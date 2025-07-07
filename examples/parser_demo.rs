use parser::parse;

pub fn main() {
    // Test some example programs with our simplified weight: rule format
    let examples = vec![
        // Single rule
        "1.5: simple rule",
        // Multiple rules
        r#"1.0: first rule
2.5: second rule
10.0: third rule"#,
        // Rules with various text content
        r#"3.14: rule with numbers 123 and symbols !@#
0.5: rule with multiple   spaces
100.0: rule with punctuation, commas; and colons: but only the first colon matters
42.0: rule with "quotes" and 'apostrophes'"#,
        // Edge cases
        "0.1: very small weight",
        "999.999: very large weight",
        // Error cases
        "-1.0: invalid negative weight",
        "1.5: valid rule\n2.0 missing colon after this",
    ];

    for (i, example) in examples.iter().enumerate() {
        println!("=== Example {} ===", i + 1);
        println!("Source:");
        println!("{}", example);

        match parse(example) {
            Ok(program) => {
                println!("✅ Parsed successfully!");
                println!("Found {} rule(s):", program.rules.len());
                for (j, rule) in program.rules.iter().enumerate() {
                    println!(
                        "  Rule {}: weight={}, text=\"{}\"",
                        j + 1,
                        rule.value.weight,
                        rule.value.text
                    );
                }
            }
            Err(e) => {
                println!("❌ Parse error: {}", e);
            }
        }
        println!();
    }
}
