use table_collection::parse;

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
                                table_collection::RuleContent::Text(text) => {
                                    println!("        [{}] Text: {:?}", i, text);
                                }
                                table_collection::RuleContent::Expression(
                                    table_collection::Expression::TableReference { table_id, modifiers },
                                ) => {
                                    if modifiers.is_empty() {
                                        println!("        [{}] Table Reference: {{#{}}}", i, table_id);
                                    } else {
                                        println!("        [{}] Table Reference with modifiers: {{#{}|{}}}", i, table_id, modifiers.join("|"));
                                    }
                                }
                                table_collection::RuleContent::Expression(
                                    table_collection::Expression::ExternalTableReference { publisher, collection, table_id, modifiers },
                                ) => {
                                    if modifiers.is_empty() {
                                        println!("        [{}] External Table Reference: {{@{}/{}#{}}}", i, publisher, collection, table_id);
                                    } else {
                                        println!("        [{}] External Table Reference with modifiers: {{@{}/{}#{}|{}}}", i, publisher, collection, table_id, modifiers.join("|"));
                                    }
                                }
                                table_collection::RuleContent::Expression(
                                    table_collection::Expression::DiceRoll { count, sides },
                                ) => {
                                    match count {
                                        Some(c) => println!("        [{}] Dice Roll: {{{}d{}}}", i, c, sides),
                                        None => println!("        [{}] Dice Roll: {{d{}}}", i, sides),
                                    }
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
