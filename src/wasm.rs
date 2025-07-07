//! WebAssembly bindings for the TBL parser and collection generator

use crate::{Collection, CollectionError, parse};
use wasm_bindgen::prelude::*;

// Set up a custom panic hook for better error messages in WASM
pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// Use wee_alloc as the global allocator for smaller binary size
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// A WASM-compatible wrapper around the TBL parser
#[wasm_bindgen]
pub struct WasmParser;

#[wasm_bindgen]
impl WasmParser {
    /// Parse TBL source code and return a JSON representation of the AST
    #[wasm_bindgen]
    pub fn parse(source: &str) -> Result<String, String> {
        set_panic_hook();

        match parse(source) {
            Ok(program) => match serde_json::to_string(&program) {
                Ok(json) => Ok(json),
                Err(e) => Err(format!("JSON serialization error: {}", e)),
            },
            Err(e) => Err(format!("Parse error: {}", e)),
        }
    }

    /// Validate TBL source code without returning the AST
    #[wasm_bindgen]
    pub fn validate(source: &str) -> bool {
        set_panic_hook();
        parse(source).is_ok()
    }
}

/// A WASM-compatible wrapper around the Collection generator
#[wasm_bindgen]
pub struct WasmCollection {
    collection: Collection,
}

#[wasm_bindgen]
impl WasmCollection {
    /// Create a new collection from TBL source code
    #[wasm_bindgen(constructor)]
    pub fn new(source: &str) -> Result<WasmCollection, String> {
        set_panic_hook();

        match Collection::new(source) {
            Ok(collection) => Ok(WasmCollection { collection }),
            Err(e) => Err(format!("Collection creation error: {}", e)),
        }
    }

    /// Generate content from a table by ID
    #[wasm_bindgen]
    pub fn generate(&mut self, table_id: &str, count: usize) -> Result<String, String> {
        match self.collection.generate(table_id, count) {
            Ok(result) => Ok(result),
            Err(e) => Err(format!("Generation error: {}", e)),
        }
    }

    /// Check if a table exists in the collection
    #[wasm_bindgen]
    pub fn has_table(&self, table_id: &str) -> bool {
        self.collection.has_table(table_id)
    }

    /// Get a list of all table IDs in the collection
    #[wasm_bindgen]
    pub fn get_table_ids(&self) -> Vec<String> {
        self.collection.get_table_ids()
    }
}

/// Utility functions for WASM
#[wasm_bindgen]
pub struct WasmUtils;

#[wasm_bindgen]
impl WasmUtils {
    /// Get the version of the parser
    #[wasm_bindgen]
    pub fn version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    /// Example TBL source code for testing
    #[wasm_bindgen]
    pub fn example_source() -> String {
        r#"#color
1.0: red
2.0: blue
3.0: green

#shape
1.0: circle
2.0: square

#item
1.0: {#color} {#shape}
2.0: big {#color} {#shape}
0.5: tiny {#shape}"#
            .to_string()
    }
}

#[cfg(test)]
#[cfg(feature = "wasm-test")]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_wasm_parser() {
        let source = r#"#test
1.0: hello world"#;

        let result = WasmParser::parse(source);
        assert!(result.is_ok());

        let json = result.unwrap();
        assert!(json.contains("hello world"));
    }

    #[wasm_bindgen_test]
    fn test_wasm_collection() {
        let source = r#"#color
1.0: red
2.0: blue"#;

        let mut collection = WasmCollection::new(source).unwrap();
        let result = collection.generate("color", 1);
        assert!(result.is_ok());

        let generated = result.unwrap();
        assert!(generated == "red" || generated == "blue");
    }

    #[wasm_bindgen_test]
    fn test_wasm_utils() {
        let version = WasmUtils::version();
        assert!(!version.is_empty());

        let example = WasmUtils::example_source();
        assert!(example.contains("#color"));
    }
}
