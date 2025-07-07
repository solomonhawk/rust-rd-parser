use table_collection::{Collection, parse};

fn main() {
    println!("Testing External Table Reference Parsing and Error Handling");
    println!("{}", "=".repeat(60));

    // Test 1: Parsing external table references
    println!("\n1. Testing external reference parsing:");
    let source_with_external = r#"
#greeting
1.0: Hello {@user/common#name}!
2.0: Hi there {@user/common#name|capitalize}!
3.0: Welcome {@admin/special#title} {@user/common#name}!
"#;

    match parse(source_with_external) {
        Ok(program) => {
            println!("✓ Successfully parsed collection with external references");

            // Print out the parsed structure
            for table in &program.tables {
                println!("  Table: {}", table.value.metadata.id);
                for (i, rule) in table.value.rules.iter().enumerate() {
                    println!("    Rule {}: weight={}", i + 1, rule.value.weight);
                    for content in &rule.value.content {
                        match content {
                            table_collection::RuleContent::Text(text) => {
                                println!("      Text: {:?}", text);
                            }
                            table_collection::RuleContent::Expression(expr) => match expr {
                                table_collection::Expression::TableReference {
                                    table_id,
                                    modifiers,
                                } => {
                                    println!(
                                        "      TableRef: {} with modifiers: {:?}",
                                        table_id, modifiers
                                    );
                                }
                                table_collection::Expression::ExternalTableReference {
                                    publisher,
                                    collection,
                                    table_id,
                                    modifiers,
                                } => {
                                    println!(
                                        "      ExternalRef: @{}/{}#{} with modifiers: {:?}",
                                        publisher, collection, table_id, modifiers
                                    );
                                }
                                table_collection::Expression::DiceRoll { count, sides } => {
                                    println!("      DiceRoll: {}d{}", count.unwrap_or(1), sides);
                                }
                            },
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to parse: {}", e);
            return;
        }
    }

    // Test 2: Collection creation should fail with missing dependency error
    println!("\n2. Testing collection creation with external references:");
    match Collection::new(source_with_external) {
        Ok(_) => {
            println!("✗ Unexpectedly succeeded - should have failed with missing dependency");
        }
        Err(e) => {
            println!("✓ Correctly failed with error: {}", e);
        }
    }

    // Test 3: Test invalid external reference syntax
    println!("\n3. Testing invalid external reference syntax:");
    let invalid_external_sources = vec![
        ("{@invalid}", "Missing collection and table parts"),
        ("{@user/}", "Missing table part"),
        ("{@/collection#table}", "Missing publisher part"),
        ("{@user/collection}", "Missing table part"),
        ("{@user collection#table}", "Space in publisher/collection"),
    ];

    for (source, description) in invalid_external_sources {
        let test_source = format!("#test\n1.0: {}", source);
        match parse(&test_source) {
            Ok(_) => {
                println!(
                    "✗ {} - Unexpectedly succeeded parsing: {}",
                    description, source
                );
            }
            Err(e) => {
                println!("✓ {} - Correctly failed: {}", description, e);
            }
        }
    }

    // Test 4: Test backwards compatibility - normal table references should still work
    println!("\n4. Testing backwards compatibility:");
    let backwards_compatible_source = r#"
#color
1.0: red
2.0: blue

#item
1.0: {#color} ball
2.0: {#color|capitalize} sphere
"#;

    match Collection::new(backwards_compatible_source) {
        Ok(mut collection) => {
            println!("✓ Backwards compatible collection created successfully");

            // Test generation
            match collection.generate("item", 3) {
                Ok(result) => {
                    println!("✓ Generated: {}", result);
                }
                Err(e) => {
                    println!("✗ Generation failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ Backwards compatible collection failed: {}", e);
        }
    }

    // Test 5: Test mixed references (internal and external)
    println!("\n5. Testing mixed internal and external references:");
    let mixed_source = r#"
#local_color
1.0: red
2.0: blue

#mixed_item
1.0: {#local_color} {@external/items#weapon}
2.0: {@external/colors#special} {#local_color} item
"#;

    match parse(mixed_source) {
        Ok(_) => {
            println!("✓ Successfully parsed mixed internal/external references");

            // This should fail at collection creation due to external dependency
            match Collection::new(mixed_source) {
                Ok(_) => println!("✗ Unexpectedly succeeded creating collection"),
                Err(e) => println!("✓ Correctly failed at collection creation: {}", e),
            }
        }
        Err(e) => {
            println!("✗ Failed to parse mixed references: {}", e);
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("External reference testing complete!");
}
