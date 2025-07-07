use table_collection::Collection;

pub fn main() {
    let source = r#"#animal
1.0: cat
1.0: dog
1.0: bird

#modifier_tests
1.0: Basic: {#animal}
1.0: Capitalize: {#animal|capitalize}
1.0: Uppercase: {#animal|uppercase}
1.0: Lowercase: {#animal|lowercase}
1.0: Indefinite: {#animal|indefinite}
1.0: Definite: {#animal|definite}
1.0: Combined: {#animal|indefinite|capitalize}
1.0: Triple: {#animal|definite|uppercase}"#;

    match Collection::new(source) {
        Ok(mut collection) => {
            println!("🎯 Testing All Table Reference Modifiers");
            println!("========================================\n");

            println!("Valid modifiers:");
            println!("• capitalize - Capitalizes the first letter");
            println!("• uppercase - Converts to uppercase");
            println!("• lowercase - Converts to lowercase");  
            println!("• indefinite - Adds 'a' or 'an' article");
            println!("• definite - Adds 'the' article");
            println!("• Multiple modifiers can be chained with '|'\n");

            for i in 1..=15 {
                match collection.generate("modifier_tests", 1) {
                    Ok(result) => println!("  {}: {}", i, result),
                    Err(e) => println!("  Error: {}", e),
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Collection creation error: {}", e);
        }
    }
}
