use parser::parse;

pub fn main() {
    // Test some example programs
    let examples = vec![
        // Variable declarations and assignments
        r#"
            let x = 42;
            let y = x + 10;
            x = y * 2;
        "#,
        // Nested expressions
        r#"
            let result = (1 + 2) * (3 - 4) / 5;
        "#,
        // If-else statements
        r#"
            if (x > 0) {
                return x;
            } else {
                if (x < 0) {
                    return -x;
                } else {
                    return 0;
                }
            }
        "#,
        // While loop with complex condition
        r#"
            while (i < 10 && j > 0) {
                i = i + 1;
                j = j - 1;
                if (i == j) {
                    return i;
                }
            }
        "#,
        // Function calls
        r#"
            let result = add(multiply(x, 2), divide(y, 3));
            print("Result:", result);
        "#,
    ];

    for (i, example) in examples.iter().enumerate() {
        println!("=== Example {} ===", i + 1);
        println!("Source:");
        println!("{}", example);

        match parse(example) {
            Ok(ast) => {
                println!("✅ Parsed successfully!");
                println!("AST: {:#?}", ast);
            }
            Err(e) => {
                println!("❌ Parse error: {}", e);
            }
        }
        println!();
    }
}
