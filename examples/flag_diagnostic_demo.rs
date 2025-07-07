use table_collection::parse;

pub fn main() {
    println!("🎯 Testing Improved Table Flag Diagnostics");
    println!("==========================================\n");

    // Test cases with invalid table flags
    let test_cases = [
        ("#table[invalid]\n1.0: test", "Single invalid flag"),
        (
            "#table[export,invalid]\n1.0: test",
            "Invalid flag after valid flag",
        ),
        (
            "#table[unknown,another]\n1.0: test",
            "Multiple invalid flags",
        ),
        ("#table[invalidflag", "Unclosed bracket with invalid flag"),
        (
            "#table[invalidverylongflagname]\n1.0: test",
            "Long invalid flag name",
        ),
    ];

    for (i, (source, description)) in test_cases.iter().enumerate() {
        println!("📝 Test {}: {}", i + 1, description);
        println!("   Source: {}", source.replace('\n', "\\n"));

        match parse(source) {
            Ok(_) => println!("   ✅ Unexpected success!"),
            Err(e) => {
                println!("   ❌ Error (showing improved diagnostic):");
                // Print the error with indentation
                for line in format!("{}", e).lines() {
                    println!("      {}", line);
                }
            }
        }
        println!();
    }

    println!("🎯 Expected Improvements:");
    println!("   • Error highlights from '[' to ']' (entire flag list)");
    println!("   • Better context for debugging invalid flag declarations");
    println!("   • Clearer visual indication of the problematic flag area");
}
