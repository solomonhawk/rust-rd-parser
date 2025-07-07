use table_collection::Collection;

pub fn main() {
    let source = r#"#animal[export]
1.0: cat
1.0: dog  
1.0: bird

#item
1.0: {#animal|indefinite}
2.0: {#animal|definite}
3.0: {#animal|capitalize}
4.0: {#animal|uppercase}
5.0: {#animal|lowercase}
6.0: {#animal|indefinite|capitalize}

#complex_item
1.0: You see {#animal|indefinite} walking
2.0: {#animal|definite|capitalize} looks at you
3.0: A {#animal|uppercase} appears!"#;

    match Collection::new(source) {
        Ok(mut collection) => {
            println!("🎯 Testing Table Reference Modifiers");
            println!("===================================\n");

            println!("Simple modifier examples:");
            for i in 1..=6 {
                match collection.generate("item", 1) {
                    Ok(result) => println!("  {}: {}", i, result),
                    Err(e) => println!("  Error: {}", e),
                }
            }

            println!("\nComplex modifier examples:");
            for i in 1..=6 {
                match collection.generate("complex_item", 1) {
                    Ok(result) => println!("  {}: {}", i, result),
                    Err(e) => println!("  Error: {}", e),
                }
            }

            println!("\nTable IDs:");
            println!("  All: {:?}", collection.get_table_ids());
            println!("  Exported: {:?}", collection.get_exported_table_ids());
        }
        Err(e) => {
            eprintln!("❌ Collection creation error: {}", e);
        }
    }
}
