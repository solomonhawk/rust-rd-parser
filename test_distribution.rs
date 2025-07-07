// Test script to verify weighted distribution is working correctly
use table_collection::Collection;

pub fn main() {
    let source = r#"#test
1.0: rare
10.0: common"#;

    let mut collection = Collection::new(source).unwrap();

    let mut rare_count = 0;
    let mut common_count = 0;
    let total_tests = 1000;

    for _ in 0..total_tests {
        let result = collection.generate("test", 1).unwrap();
        if result == "rare" {
            rare_count += 1;
        } else if result == "common" {
            common_count += 1;
        }
    }

    println!("Results from {} tests:", total_tests);
    println!(
        "Rare (weight 1.0): {} times ({:.1}%)",
        rare_count,
        (rare_count as f64 / total_tests as f64) * 100.0
    );
    println!(
        "Common (weight 10.0): {} times ({:.1}%)",
        common_count,
        (common_count as f64 / total_tests as f64) * 100.0
    );

    // Expected ratio: common should appear ~10x more often than rare
    // With weights 1.0 and 10.0, expected percentages are ~9.1% and ~90.9%
    let expected_rare_percent = 1.0 / 11.0 * 100.0; // ~9.1%
    let expected_common_percent = 10.0 / 11.0 * 100.0; // ~90.9%

    println!("Expected rare: {:.1}%", expected_rare_percent);
    println!("Expected common: {:.1}%", expected_common_percent);
}
