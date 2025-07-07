# Rust TBL (Table) Language Parser with Enhanced Diagnostics

A hand-written recursive descent parser in Rust for the TBL (Table) language, featuring a sophisticated diagnostic system with rich error reporting and contextual suggestions.

## Overview

This parser handles the TBL (Table) language format:

```
#table_id[flags]
<weight>: <rule>
<weight>: <rule>
...

#another_table[export]
<weight>: <rule>
...
```

Where:
- `table_id` is an identifier for the table
- `flags` are optional metadata (currently supports `export`)
- `weight` is a positive floating point number
- `rule` is text content until newline
- Table references can include modifiers: `{#table|modifier1|modifier2}`

### Table Reference Modifiers

Table references support modifiers that transform the generated content:

```
#animal
1.0: cat
1.0: dog
1.0: bird

#item
1.0: {#animal|indefinite}         // "a cat", "a dog", "an elephant"
2.0: {#animal|definite}           // "the cat", "the dog", "the bird"
3.0: {#animal|capitalize}         // "Cat", "Dog", "Bird"
4.0: {#animal|uppercase}          // "CAT", "DOG", "BIRD"
5.0: {#animal|lowercase}          // "cat", "dog", "bird"
6.0: {#animal|indefinite|capitalize}  // "A cat", "A dog", "An elephant"
```

**Available Modifiers:**
- `indefinite` - Adds "a" or "an" based on first letter
- `definite` - Adds "the" prefix
- `capitalize` - Capitalizes the first letter
- `uppercase` - Converts to all uppercase
- `lowercase` - Converts to all lowercase

Modifiers can be chained using the pipe `|` separator and are applied in order.

## Key Features

### 🔧 **Architectural Separation**
- **Diagnostic Collection**: Pure structured data about errors
- **Diagnostic Formatting**: Customizable rendering of error messages
- **Clean Boundaries**: Error data collection is separate from presentation

### 📍 **Rich Error Reporting**
- Exact line and column positions
- Visual error pointers
- Context-aware suggestions
- Multiple formatting options

### 🎯 **Enhanced User Experience**
- Helpful suggestions for common mistakes
- Clear explanations of what went wrong
- Beautiful error formatting with emojis and visual guides

## Architecture

### Core Components

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Lexer/Parser  │───▶│ DiagnosticData  │───▶│  Formatter      │
│                 │    │   (Structured)  │    │  (Presentation) │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

1. **`Diagnostic`** - Structured error data with source location and context
2. **`DiagnosticCollector`** - Gathers context and creates diagnostic data
3. **`DiagnosticFormatter`** - Renders diagnostics into human-readable output
4. **`LexError`/`ParseError`** - Error types containing diagnostic information

### Error Flow

```rust
Source Code → Lexer/Parser → Diagnostic Data → Formatter → User Output
```

## Usage

### Basic Parsing

```rust
use table_collection::parse;

let source = r#"#shapes
1.5: circle
2.0: square

#colors[export]
1.0: red
3.0: blue"#;

match parse(source) {
    Ok(program) => {
        println!("Parsed {} tables", program.tables.len());
        for table in &program.tables {
            println!("Table '{}' has {} rules", table.value.metadata.id, table.value.rules.len());
        }
    },
    Err(e) => println!("Error:\n{}", e),
}
```

### Advanced Diagnostic Usage

```rust
use table_collection::{DiagnosticFormatter, DiagnosticCollector};

// Custom formatting
let formatter = DiagnosticFormatter::new()
    .with_colors(false)
    .with_suggestions(true);

// Manual diagnostic creation
let collector = DiagnosticCollector::new(source.to_string());
let diagnostic = collector.parse_error(10, "Custom error".to_string())
    .with_suggestion("Try this instead".to_string());

println!("{}", formatter.format(&diagnostic));
```

## Error Examples

### Missing Table Declaration
```
❌ Expected '#' to start table declaration
   ┌─ line 1:1
   │
  1 │ 1.5: missing table declaration
   │ ^
   │
   = 💡 suggestion: Expected Expected '#' to start table declaration
```

### Invalid Character
```
❌ Invalid character '-'
   ┌─ line 2:1
   │
  2 │ -1.0: negative weight
   │ ^
   │
   = 💡 suggestion: Negative numbers are not allowed. Use positive weights like 1.0, 2.5
```

### Missing Colon
```
❌ Expected ':' after weight
   ┌─ line 2:5
   │
  2 │ 1.5 missing colon after weight
   │     ^
   │
   = 💡 suggestion: Only numbers, colons, and rule text are allowed in this language
```

### Invalid Weight
```
❌ Weight must be positive, but got 0
   ┌─ line 1:1
   │
  1 │ 0: zero weight
   │ ^
   │
   = 💡 suggestion: Try using a positive number like 1.0, 2.5, or 10
```

## API Reference

### Core Types

- **`Program`** - Root AST node containing tables
- **`Table`** - Table with metadata and rules
- **`TableMetadata`** - Table identifier and flags (export)
- **`Rule`** - Single rule with weight and text
- **`Token`** - Lexical token with type and span information

### Diagnostic System

- **`Diagnostic`** - Structured error with location and message
- **`DiagnosticCollector`** - Creates diagnostics from source positions
- **`DiagnosticFormatter`** - Customizable error formatting
- **`SourceLocation`** - Line/column position information

### Error Types

- **`LexError`** - Lexical analysis errors (optimized with `Box<Diagnostic>`)
- **`ParseError`** - Parsing errors (optimized with `Box<Diagnostic>`)
- **`ParseResult<T>`** - Result type for parsing operations

## Performance Optimizations

The error types have been optimized to reduce memory usage:
- Large diagnostic data is boxed to keep error variants small
- This reduces stack usage and improves performance when errors are not the common path
- All clippy warnings about large error variants have been resolved

## Examples

Run the demos to see the diagnostic system in action:

```bash
# Basic parser demo with error examples
cargo run --example parser_demo

# Advanced diagnostic usage
cargo run --example diagnostic_demo
```

## Testing

```bash
cargo test
```

All tests pass and demonstrate that the architectural refactoring maintains full compatibility with existing functionality.

## Benefits of the New Architecture

1. **Modularity**: Clear separation of concerns between error collection and formatting
2. **Testability**: Structured diagnostic data can be easily tested and validated
3. **Customization**: Error formatting can be customized without changing core logic
4. **Extensibility**: Easy to add new diagnostic types and formatting options
5. **User Experience**: Rich, helpful error messages guide users to solutions

## Design Principles

- **Structured First**: Error data is collected as structured information before formatting
- **Context Rich**: Every error includes full source context and helpful suggestions
- **User Focused**: Error messages are designed to help users understand and fix problems
- **Performance Conscious**: Diagnostic collection is efficient and only formats when needed
- **Type Safe**: Strong typing ensures diagnostic consistency and prevents runtime errors
