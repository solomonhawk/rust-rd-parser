use parser::parse;

pub fn main() {
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
3.0: small {#shape}
4.0: tiny red {#shape}"#;

    match parse(source) {
        Ok(program) => {
            println!("✅ Successfully parsed {} tables!", program.tables.len());

            for table in &program.tables {
                println!("\n📋 Table: #{}", table.value.metadata.id);
                if table.value.metadata.export {
                    println!("   (exported)");
                }

                for rule in &table.value.rules {
                    println!("   {}: {}", rule.value.weight, rule.value.content_text());

                    // Show the internal structure for the item table
                    if table.value.metadata.id == "item" {
                        println!("      Content breakdown:");
                        for (i, content) in rule.value.content.iter().enumerate() {
                            match content {
                                parser::RuleContent::Text(text) => {
                                    println!("        [{}] Text: {:?}", i, text);
                                }
                                parser::RuleContent::Expression(
                                    parser::Expression::TableReference { table_id },
                                ) => {
                                    println!("        [{}] Expression: {{#{}}}", i, table_id);
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Parse error: {}", e);
        }
    }
}
