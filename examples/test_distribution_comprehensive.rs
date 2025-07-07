// Test script to verify edge cases in weighted distribution
use table_collection::Collection;

fn test_distribution(source: &str, expected_ratios: &[(&str, f64)], test_name: &str) {
    let mut collection = Collection::new(source).unwrap();

    let mut counts = std::collections::HashMap::new();
    let total_tests = 10000; // More tests for better accuracy

    for _ in 0..total_tests {
        let result = collection.generate("test", 1).unwrap();
        *counts.entry(result).or_insert(0) += 1;
    }

    println!("\n{}", test_name);
    println!("Results from {} tests:", total_tests);

    let total_weight: f64 = expected_ratios.iter().map(|(_, w)| w).sum();

    for (expected_result, expected_weight) in expected_ratios {
        let actual_count = *counts.get(*expected_result).unwrap_or(&0);
        let actual_percent = (actual_count as f64 / total_tests as f64) * 100.0;
        let expected_percent = (expected_weight / total_weight) * 100.0;

        println!(
            "{}: {} times ({:.1}%) - expected {:.1}%",
            expected_result, actual_count, actual_percent, expected_percent
        );

        // Check if the actual percentage is within a reasonable range of expected
        let diff = (actual_percent - expected_percent).abs();
        if diff > 2.0 {
            // Allow 2% tolerance
            println!("  ⚠️  Warning: Deviation of {:.1}% from expected", diff);
        }
    }
}

pub fn main() {
    // Test 1: Basic case
    test_distribution(
        r#"#test
1.0: rare
10.0: common"#,
        &[("rare", 1.0), ("common", 10.0)],
        "Test 1: Basic weighted distribution (1:10 ratio)",
    );

    // Test 2: Equal weights
    test_distribution(
        r#"#test
5.0: option1
5.0: option2"#,
        &[("option1", 5.0), ("option2", 5.0)],
        "Test 2: Equal weights (should be 50/50)",
    );

    // Test 3: Many options with different weights
    test_distribution(
        r#"#test
1.0: very_rare
2.0: rare
5.0: uncommon
10.0: common
2.0: rare2"#,
        &[
            ("very_rare", 1.0),
            ("rare", 2.0),
            ("uncommon", 5.0),
            ("common", 10.0),
            ("rare2", 2.0),
        ],
        "Test 3: Multiple options with varied weights",
    );

    // Test 4: Very small weights
    test_distribution(
        r#"#test
0.1: tiny
0.9: small
9.0: big"#,
        &[("tiny", 0.1), ("small", 0.9), ("big", 9.0)],
        "Test 4: Small decimal weights",
    );

    // Test 5: Single option (edge case)
    test_distribution(
        r#"#test
1.0: only_option"#,
        &[("only_option", 1.0)],
        "Test 5: Single option (should always be selected)",
    );
}
