use table_collection::{Collection, parse};

pub fn main() {
    let source = r#"#animal
1.0: cat

#test
1.0: {#animal|capitalize}
1.0: {#animal|indefinite|uppercase}"#;

    println!("Testing parsing with valid modifiers...");
    match parse(source) {
        Ok(program) => {
            println!("✅ Parsing succeeded!");
            println!("Program has {} tables", program.tables.len());

            match Collection::new(source) {
                Ok(mut collection) => {
                    println!("✅ Collection creation succeeded!");
                    match collection.generate("test", 5) {
                        Ok(result) => println!("✅ Generation succeeded: {}", result),
                        Err(e) => println!("❌ Generation failed: {}", e),
                    }
                }
                Err(e) => println!("❌ Collection creation failed: {}", e),
            }
        }
        Err(e) => println!("❌ Parsing failed: {}", e),
    }
}
