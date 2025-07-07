use std::time::Instant;
use table_collection::Collection;

fn main() {
    println!("🚀 TBL Collection Performance Demo");
    println!("==================================");
    println!("Testing performance improvements from parse-time optimizations\n");

    // Create a table with many rules to demonstrate the performance difference
    let source = r#"#performance_test
1.0: result_1
2.0: result_2
3.0: result_3
4.0: result_4
5.0: result_5
6.0: result_6
7.0: result_7
8.0: result_8
9.0: result_9
10.0: result_10
1.5: result_11
2.5: result_12
3.5: result_13
4.5: result_14
5.5: result_15
6.5: result_16
7.5: result_17
8.5: result_18
9.5: result_19
10.5: result_20
1.1: result_21
2.1: result_22
3.1: result_23
4.1: result_24
5.1: result_25
6.1: result_26
7.1: result_27
8.1: result_28
9.1: result_29
10.1: result_30

#compound_test
1.0: {#performance_test} item A
2.0: {#performance_test} item B
3.0: {#performance_test} item C
4.0: Complex {#performance_test} with {#performance_test}
5.0: Multiple {#performance_test} references {#performance_test} here"#;

    match Collection::new(source) {
        Ok(mut collection) => {
            println!("✅ Collection created successfully");
            println!("📊 Table 'performance_test' has 30 rules");
            println!("📊 Table 'compound_test' has 5 rules with table references\n");

            // Test simple generation performance
            println!("🔬 Performance Test: Simple Generation");
            let iterations = 10_000;

            let start = Instant::now();
            for _ in 0..iterations {
                let _ = collection.generate("performance_test", 1);
            }
            let duration = start.elapsed();

            println!("   Generated {} items in {:?}", iterations, duration);
            println!(
                "   Average: {:.2}μs per generation",
                duration.as_micros() as f64 / iterations as f64
            );

            // Test compound generation performance
            println!("\n🔬 Performance Test: Compound Generation (with table references)");

            let start = Instant::now();
            for _ in 0..iterations {
                let _ = collection.generate("compound_test", 1);
            }
            let duration = start.elapsed();

            println!(
                "   Generated {} compound items in {:?}",
                iterations, duration
            );
            println!(
                "   Average: {:.2}μs per generation",
                duration.as_micros() as f64 / iterations as f64
            );

            // Show some sample outputs
            println!("\n📋 Sample Outputs:");
            for i in 1..=5 {
                match collection.generate("compound_test", 1) {
                    Ok(result) => println!("   {}: {}", i, result),
                    Err(e) => println!("   Error: {}", e),
                }
            }

            println!("\n🎯 Optimizations Applied:");
            println!("   ✅ Pre-computed cumulative weights (parse-time)");
            println!("   ✅ Cached total weights (parse-time)");
            println!("   ✅ Binary search for rule selection (O(log n) vs O(n))");
            println!("   ✅ No weight recalculation during generation");
        }
        Err(e) => {
            println!("❌ Failed to create collection: {}", e);
        }
    }
}
