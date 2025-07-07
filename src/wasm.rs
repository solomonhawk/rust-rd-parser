//! WebAssembly bindings for the TBL parser and collection generator

use crate::{Collection, parse};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// Set up a custom panic hook for better error messages in WASM
pub fn set_panic_hook() {
    #[cfg(feature = "wasm")]
    console_error_panic_hook::set_once();
}

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

    /// Parse TBL source code and return detailed diagnostics
    #[wasm_bindgen]
    pub fn parse_with_diagnostics(source: &str) -> String {
        set_panic_hook();

        let result = match parse(source) {
            Ok(program) => {
                let ast_json = match serde_json::to_string(&program) {
                    Ok(json) => Some(json),
                    Err(e) => {
                        let diagnostic = WasmDiagnostic {
                            message: format!("JSON serialization error: {}", e),
                            severity: "error".to_string(),
                            line: 0,
                            column: 0,
                            end_line: 0,
                            end_column: 0,
                            source: "".to_string(),
                        };

                        return serde_json::to_string(&WasmParseResult {
                            success: false,
                            ast_json: None,
                            diagnostics: vec![diagnostic],
                        })
                        .unwrap_or_else(|_| "{}".to_string());
                    }
                };

                WasmParseResult {
                    success: true,
                    ast_json,
                    diagnostics: vec![], // No diagnostics for successful parse
                }
            }
            Err(parse_error) => {
                // Convert parse error to diagnostic with proper position info
                let diagnostic = match &parse_error {
                    crate::errors::ParseError::UnexpectedToken { diagnostic, .. }
                    | crate::errors::ParseError::UnexpectedEof { diagnostic, .. }
                    | crate::errors::ParseError::InvalidCharacter { diagnostic, .. }
                    | crate::errors::ParseError::InvalidNumber { diagnostic, .. } => {
                        // Extract position information from the diagnostic
                        let location = &diagnostic.location;
                        WasmDiagnostic {
                            message: parse_error.to_string(),
                            severity: "error".to_string(),
                            line: location.line as u32,
                            column: location.column as u32,
                            end_line: location.line as u32, // For now, same line
                            end_column: (location.column + 1) as u32, // Assume single character for now
                            source: diagnostic.source_line.clone(),
                        }
                    }
                };

                WasmParseResult {
                    success: false,
                    ast_json: None,
                    diagnostics: vec![diagnostic],
                }
            }
        };

        // Serialize the result to JSON for JavaScript consumption
        serde_json::to_string(&result).unwrap_or_else(|_| {
            r#"{"success": false, "ast_json": null, "diagnostics": [{"message": "Failed to serialize parse result", "severity": "error", "line": 0, "column": 0, "end_line": 0, "end_column": 0, "source": ""}]}"#.to_string()
        })
    }

    /// Quick validation with basic diagnostic info
    #[wasm_bindgen]
    pub fn validate_with_diagnostics(source: &str) -> String {
        set_panic_hook();

        let diagnostics = match parse(source) {
            Ok(_) => vec![], // No diagnostics for successful parse
            Err(parse_error) => {
                // Convert parse error to diagnostic with proper position info
                let diagnostic = match &parse_error {
                    crate::errors::ParseError::UnexpectedToken { diagnostic, .. }
                    | crate::errors::ParseError::UnexpectedEof { diagnostic, .. }
                    | crate::errors::ParseError::InvalidCharacter { diagnostic, .. }
                    | crate::errors::ParseError::InvalidNumber { diagnostic, .. } => {
                        // Extract position information from the diagnostic
                        let location = &diagnostic.location;
                        WasmDiagnostic {
                            message: parse_error.to_string(),
                            severity: "error".to_string(),
                            line: location.line as u32,
                            column: location.column as u32,
                            end_line: location.line as u32, // For now, same line
                            end_column: (location.column + 1) as u32, // Assume single character for now
                            source: diagnostic.source_line.clone(),
                        }
                    }
                };
                vec![diagnostic]
            }
        };

        let result = WasmParseResult {
            success: diagnostics.is_empty(),
            ast_json: None,
            diagnostics,
        };

        serde_json::to_string(&result).unwrap_or_else(|_| {
            r#"{"success": false, "ast_json": null, "diagnostics": []}"#.to_string()
        })
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

    /// Get a list of exported table IDs in the collection
    #[wasm_bindgen]
    pub fn get_exported_table_ids(&self) -> Vec<String> {
        self.collection.get_exported_table_ids()
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

/// Diagnostic information for language server support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmDiagnostic {
    /// The error/warning message
    pub message: String,
    /// The severity level (error, warning, info)
    pub severity: String,
    /// The line number (0-based)
    pub line: u32,
    /// The column number (0-based)
    pub column: u32,
    /// The end line number (0-based)
    pub end_line: u32,
    /// The end column number (0-based)
    pub end_column: u32,
    /// The source code that caused the diagnostic
    pub source: String,
}

/// Parse result with diagnostics for language server
#[derive(Debug, Serialize, Deserialize)]
pub struct WasmParseResult {
    /// Whether parsing was successful
    pub success: bool,
    /// JSON representation of the AST (if successful)
    pub ast_json: Option<String>,
    /// List of diagnostics (errors, warnings, etc.)
    pub diagnostics: Vec<WasmDiagnostic>,
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

        let result = Wasmtable_collection::parse(source);
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
