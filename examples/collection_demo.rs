use parser::{Collection, CollectionError};

pub fn main() {
    let source = r#"#color
1.0: red
2.0: blue
3.0: green

#shape
1.0: circle
2.0: square

#size
10.0: tiny
5.0: small
3.0: medium
1.0: large

#item
1.0: {#color} {#shape}
2.0: {#size} {#color} {#shape}
1.5: beautiful {#color} {#shape}
0.5: mysterious {#size} {#shape}"#;

    match Collection::new(source) {
        Ok(mut collection) => {
            println!("🎲 TBL Collection Demo");
            println!("=====================\n");

            // Generate single items
            println!("Single generations:");
            for i in 1..=5 {
                match collection.generate("item", 1) {
                    Ok(result) => println!("  {}: {}", i, result),
                    Err(e) => println!("  Error: {}", e),
                }
            }

            println!("\nMultiple generations:");
            // Generate multiple items at once
            match collection.generate("color", 5) {
                Ok(result) => println!("  Colors (5): {}", result),
                Err(e) => println!("  Error: {}", e),
            }

            match collection.generate("shape", 3) {
                Ok(result) => println!("  Shapes (3): {}", result),
                Err(e) => println!("  Error: {}", e),
            }

            match collection.generate("item", 10) {
                Ok(result) => println!("  Items (10): {}", result),
                Err(e) => println!("  Error: {}", e),
            }

            println!("\nError handling:");
            // Test error handling
            match collection.generate("nonexistent", 1) {
                Ok(result) => println!("  Unexpected success: {}", result),
                Err(CollectionError::TableNotFound(table_id)) => {
                    println!("  ✓ Correctly caught missing table: '{}'", table_id);
                }
                Err(e) => println!("  Unexpected error: {}", e),
            }

            println!("\nDemonstrating weighted distribution:");
            println!("Size table has weights: tiny(10), small(5), medium(3), large(1)");
            println!("So we should see more tiny/small than medium/large:");
            match collection.generate("size", 20) {
                Ok(result) => {
                    println!("  Sizes (20): {}", result);
                    // Count occurrences
                    let tiny_count = result.matches("tiny").count();
                    let small_count = result.matches("small").count();
                    let medium_count = result.matches("medium").count();
                    let large_count = result.matches("large").count();

                    println!(
                        "  Distribution: tiny: {}, small: {}, medium: {}, large: {}",
                        tiny_count, small_count, medium_count, large_count
                    );
                }
                Err(e) => println!("  Error: {}", e),
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to create collection: {}", e);
        }
    }
}
