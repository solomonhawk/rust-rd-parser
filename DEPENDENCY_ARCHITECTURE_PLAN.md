# Table Collection Dependencies - Architecture Plan

## Overview

We need to support **external table collection dependencies** with the syntax:
```tbl
#foo[export]
1: {@username/collection}
```

Where `@username` refers to the publisher of another table collection and `collection` is the ID of that collection.

## Current Architecture Analysis

### Current System
- **Single Collection**: One `Collection` contains multiple tables from a single source
- **Table References**: `{#table_id}` references only work within the same collection
- **Validation**: Happens during `Collection::new()` - all references must exist
- **Parse → Validate → Generate**: Linear pipeline for single collection

### Current Reference Resolution
```rust
// In Collection::validate_table_references()
for rule in &table.rules {
    if let RuleContent::Expression(Expression::TableReference { table_id, .. }) = content {
        if !tables.contains_key(ref_id) {
            return Err(CollectionError::InvalidTableReference { /* ... */ });
        }
    }
}
```

## Proposed Architecture

### 1. Dependency Resolution Strategy

**JavaScript-Driven Dependency Resolution** ✅ **RECOMMENDED**
- JavaScript caller resolves all dependencies and fetches sources
- JavaScript passes all collection sources to Rust as a batch
- Rust handles namespace collision resolution and merging
- Rust validates and generates from the unified collection

**Why JavaScript-Driven?**
- Keeps Rust library purely functional (no async/database dependencies)
- JavaScript can handle complex dependency resolution, caching, versioning
- Rust focuses on parsing, validation, and generation performance
- Better separation of concerns and testability
- Easier to integrate with different database/storage systems

### 2. Table ID Collision Resolution

#### Namespacing Strategy
To handle table ID collisions across different publishers, we'll use **automatic namespacing**:

```rust
// Internal representation after merging
#[derive(Debug, Clone, PartialEq)]
pub struct NamespacedTableId {
    pub namespace: Option<String>, // None for primary collection
    pub table_id: String,
}

// Examples:
// Primary collection: "weapons" -> NamespacedTableId { namespace: None, table_id: "weapons" }
// External: "@rpg/medieval#sword" -> NamespacedTableId { namespace: Some("rpg.medieval"), table_id: "sword" }
```

#### Collision Resolution Rules
1. **Primary collection tables**: Keep original names (no namespace)
2. **External collections**: Namespace as `{publisher}.{collection}.{table_id}`
3. **Conflict resolution**: If primary has "sword" and external has "sword", external becomes "rpg.medieval.sword"
4. **Reference resolution**: External references automatically map to namespaced tables

### 3. New AST Structure

#### External Table Reference
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    // Existing
    TableReference { table_id: String, modifiers: Vec<String> },
    DiceRoll { count: Option<u32>, sides: u32 },

    // NEW: External collection reference
    ExternalTableReference {
        publisher: String,      // @username
        collection: String,     // collection name
        table_id: String,       // table within that collection
        modifiers: Vec<String>, // same modifiers as internal refs
    },
}
```

#### Collection Source Input
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct CollectionSource {
    pub publisher: String,    // "rpg", "username", etc.
    pub collection: String,   // "medieval", "magic", etc.
    pub source: String,       // TBL source code
    pub is_primary: bool,     // true for the main collection
}

#[derive(Debug, Clone, PartialEq)]
pub struct CollectionInput {
    pub sources: Vec<CollectionSource>,
}
```

#### Internal Namespaced Tables
```rust
#[derive(Debug, Clone)]
struct NamespacedTable {
    pub original_id: String,           // Original table ID from source
    pub namespaced_id: String,        // Final namespaced ID used internally
    pub namespace: Option<String>,    // None for primary, Some("publisher.collection") for external
    pub table: OptimizedTable,       // The actual table data
}
```

### 4. JavaScript-Driven Pipeline

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   JavaScript    │───▶│   JavaScript    │───▶│   JavaScript    │
│ Parse Primary & │    │ Resolve Deps    │    │ Fetch All Deps  │
│ Extract Deps    │    │ (Recursive)     │    │ from Database   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                                        │
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│      Rust       │◀───│      Rust       │◀───│   JavaScript    │
│ Validate Merged │    │ Parse & Merge   │    │ Pass All Sources│
│ Collection      │    │ with Namespacing│    │ to Rust         │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

#### Responsibility Split
- **JavaScript**: Dependency resolution, database queries, caching, circular dependency detection
- **Rust**: Parsing, namespace collision resolution, validation, generation

### 5. Core Components

#### Rust: CollectionMerger
```rust
pub struct CollectionMerger;

impl CollectionMerger {
    /// Merge multiple collection sources with automatic namespacing
    pub fn merge_collections(input: CollectionInput) -> CollectionResult<MergedCollectionResult> {
        // 1. Parse all sources
        // 2. Extract table lists and detect conflicts
        // 3. Apply namespacing rules
        // 4. Transform external references to internal references
        // 5. Generate merged source
        // 6. Return namespacing info for JavaScript
    }
}

#[derive(Debug, Clone)]
pub struct MergedCollectionResult {
    pub merged_source: String,                    // Combined TBL source with namespaced tables
    pub namespace_map: HashMap<String, String>,  // Original ID -> Namespaced ID mapping
    pub primary_tables: Vec<String>,             // Table IDs from primary collection
    pub external_tables: Vec<ExternalTableInfo>, // Info about external tables
}

#[derive(Debug, Clone)]
pub struct ExternalTableInfo {
    pub original_ref: String,      // "@rpg/medieval#sword"
    pub namespaced_id: String,     // "rpg.medieval.sword"
    pub publisher: String,         // "rpg"
    pub collection: String,        // "medieval"
    pub table_id: String,          // "sword"
}
```

#### JavaScript: Dependency Resolution
```javascript
class DependencyResolver {
    constructor(database) {
        this.database = database;
        this.cache = new Map();
    }

    async resolveDependencies(primarySource) {
        // 1. Parse primary source to extract external references
        const deps = this.extractDependencies(primarySource);

        // 2. Recursively resolve all dependencies
        const resolvedSources = await this.fetchAllDependencies(deps);

        // 3. Build CollectionInput for Rust
        return {
            sources: [
                {
                    publisher: "primary",
                    collection: "main",
                    source: primarySource,
                    is_primary: true
                },
                ...resolvedSources
            ]
        };
    }

    extractDependencies(source) {
        // Extract all {@publisher/collection#table} references
        const regex = /{@([^/]+)\/([^#}|]+)#[^}|]+/g;
        const dependencies = new Set();

        let match;
        while ((match = regex.exec(source)) !== null) {
            dependencies.add(`${match[1]}/${match[2]}`);
        }

        return Array.from(dependencies);
    }

    async fetchAllDependencies(deps, visited = new Set()) {
        const sources = [];

        for (const dep of deps) {
            if (visited.has(dep)) {
                throw new Error(`Circular dependency detected: ${dep}`);
            }

            visited.add(dep);
            const [publisher, collection] = dep.split('/');

            // Fetch from database with caching
            const source = await this.fetchCollection(publisher, collection);
            sources.push({
                publisher,
                collection,
                source,
                is_primary: false
            });

            // Recursively resolve dependencies
            const subDeps = this.extractDependencies(source);
            const subSources = await this.fetchAllDependencies(subDeps, new Set(visited));
            sources.push(...subSources);
        }

        return sources;
    }
}
```

### 5. Parser Updates

#### Lexer Changes
```rust
// Add new token types
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Existing...

    // NEW: External reference tokens
    ExternalRef,    // @username/collection
    At,            // @
    Slash,         // /
}
```

#### Parser Changes
```rust
impl Parser {
    fn parse_expression(&mut self) -> ParseResult<Expression> {
        if self.check(TokenType::LeftBrace) {
            if self.peek_external_reference() {
                self.parse_external_table_reference()  // NEW
            } else {
                self.parse_table_reference()           // Existing
            }
        }
        // ...
    }

    fn parse_external_table_reference(&mut self) -> ParseResult<Expression> {
        // Parse: {@username/collection#table|modifier1|modifier2}
        // ...
    }
}
```

### 6. Collection Updates

#### New Collection Creation API
```rust
impl Collection {
    // Existing: for single collection
    pub fn new(source: &str) -> CollectionResult<Self> { /* ... */ }

    // NEW: for collection with dependencies (called from WASM)
    pub fn new_with_sources(input: CollectionInput) -> CollectionResult<Self> {
        // 1. Merge all sources with namespacing
        let merged_result = CollectionMerger::merge_collections(input)?;

        // 2. Parse merged source using existing logic
        Self::new(&merged_result.merged_source)
    }
}
```

#### WASM Interface Updates
```rust
#[wasm_bindgen]
impl WasmCollection {
    /// Create a new collection from multiple sources with dependencies
    #[wasm_bindgen(constructor)]
    pub fn new_with_dependencies(sources_json: &str) -> Result<WasmCollection, String> {
        let input: CollectionInput = serde_json::from_str(sources_json)
            .map_err(|e| format!("Invalid sources JSON: {}", e))?;

        match Collection::new_with_sources(input) {
            Ok(collection) => Ok(WasmCollection { collection }),
            Err(e) => Err(format!("Collection creation error: {}", e)),
        }
    }
}
```

### 7. Error Handling

#### New Error Types
```rust
#[derive(Error, Debug)]
pub enum NamespaceError {
    #[error("Table ID collision that cannot be resolved: {table_id}")]
    UnresolvableCollision { table_id: String },

    #[error("Invalid external reference: {reference}")]
    InvalidExternalReference { reference: String },

    #[error("External table not found in provided sources: @{publisher}/{collection}#{table}")]
    ExternalTableNotFound { publisher: String, collection: String, table: String },
}

#[derive(Error, Debug)]
pub enum CollectionError {
    // Existing errors...

    // NEW: Namespace-related errors
    #[error("Namespace error: {0}")]
    NamespaceError(#[from] NamespaceError),
}
```

#### JavaScript Error Handling
```javascript
// JavaScript handles these error types:
class DependencyError extends Error {
    constructor(message, type, details = {}) {
        super(message);
        this.type = type; // 'not_found', 'circular', 'network', etc.
        this.details = details;
    }
}

// Usage:
try {
    const resolved = await dependencyResolver.resolveDependencies(source);
    const collection = new WasmCollection.new_with_dependencies(JSON.stringify(resolved));
} catch (error) {
    if (error instanceof DependencyError && error.type === 'circular') {
        // Handle circular dependency
    } else if (error.message.includes("Collection creation error")) {
        // Handle Rust-side errors (parsing, validation, etc.)
    }
}
```

### 8. Example Usage Flow

#### JavaScript Side (Dependency Resolution)
```javascript
// 1. User provides primary collection source
const primarySource = `
#weapons[export]
1: {@rpg/medieval#sword}
2: {@rpg/magic#staff|capitalize}

#character
1: warrior with {#weapons|indefinite}
`;

// 2. JavaScript resolves all dependencies
const resolver = new DependencyResolver(database);
const collectionInput = await resolver.resolveDependencies(primarySource);

// 3. Pass to Rust for parsing and validation
const collection = new WasmCollection.new_with_dependencies(
    JSON.stringify(collectionInput)
);

// 4. Generate content
const result = collection.generate("character", 1);
console.log(result); // "warrior with a steel sword"
```

#### Rust Side (Merging and Namespacing)
```rust
// Input: CollectionInput with sources:
// - Primary: #weapons, #character
// - rpg/medieval: #sword, #bow
// - rpg/magic: #staff, #wand

// After merging with namespacing:
let merged_source = `
// From rpg/medieval (namespaced)
#rpg.medieval.sword
1: iron sword
2: steel sword

#rpg.medieval.bow
1: short bow
2: long bow

// From rpg/magic (namespaced)
#rpg.magic.staff
1: oak staff
2: crystal staff

// From primary (unchanged)
#weapons[export]
1: {#rpg.medieval.sword}          // External ref resolved
2: {#rpg.magic.staff|capitalize}  // External ref resolved

#character
1: warrior with {#weapons|indefinite}
`;
```

## Implementation Strategy

### Phase 1: Core Parsing Support
1. ✅ Update AST with `ExternalTableReference`
2. ✅ Update lexer to recognize `@username/collection#table` syntax
3. ✅ Update parser to handle external references
4. ✅ Add basic error types

### Phase 2: Namespace & Merging Framework
1. ✅ Create `CollectionInput` and related structures
2. ✅ Implement `CollectionMerger` for namespace collision resolution
3. ✅ Add external reference transformation logic
4. ✅ Create merged collection structure

### Phase 3: WASM Integration
1. ✅ Update WASM interface to accept multiple sources
2. ✅ Add JSON serialization for `CollectionInput`
3. ✅ Update Collection API for multi-source creation
4. ✅ Add comprehensive error handling

### Phase 4: JavaScript Framework
1. ✅ Implement JavaScript dependency resolver
2. ✅ Add caching and circular dependency detection
3. ✅ Create database integration layer
4. ✅ Performance optimizations and testing

## Example Usage Flow

### JavaScript API Usage
```javascript
// High-level API that handles everything
class TableCollectionManager {
    constructor(database) {
        this.resolver = new DependencyResolver(database);
    }

    async createCollection(primarySource) {
        // 1. Resolve dependencies
        const input = await this.resolver.resolveDependencies(primarySource);

        // 2. Create Rust collection
        const collection = new WasmCollection.new_with_dependencies(
            JSON.stringify(input)
        );

        return new ManagedCollection(collection, input);
    }
}

// Usage:
const manager = new TableCollectionManager(database);
const collection = await manager.createCollection(userSource);
const result = collection.generate("character", 1);
```

## Example Usage

### Input Collection with Dependencies
```tbl
#weapons[export]
1: {@rpg/medieval#sword}
2: {@rpg/medieval#bow}
1: {@rpg/magic#staff|capitalize}

#character
1: warrior with {@username/items#weapon|indefinite}
```

### Dependency Resolution Process
1. **Parse Primary**: Extract `@rpg/medieval` and `@rpg/magic` dependencies
2. **Fetch Dependencies**: Resolve from database
3. **Parse Dependencies**: Extract their tables and potential sub-dependencies
4. **Merge Sources**: Combine all into unified source
5. **Validate**: Check all references resolve (including external ones)
6. **Generate**: Use unified collection for generation

### Merged Result (Internal)
```tbl
# From @rpg/medieval
#sword
1: iron sword
2: steel sword

#bow
1: short bow
2: long bow

# From @rpg/magic
#staff
1: oak staff
2: crystal staff

# From primary collection
#weapons[export]
1: {#sword}      # Resolved to internal reference
2: {#bow}        # Resolved to internal reference
1: {#staff|capitalize}

#character
1: warrior with {#weapons|indefinite}
```

## Benefits of This Architecture

1. **Clean Separation**: JavaScript handles I/O and async logic, Rust handles parsing and generation
2. **No Async Rust**: Keeps Rust library purely functional and easier to test
3. **Database Agnostic**: JavaScript can integrate with any database or storage system
4. **Performance**: Reuses existing optimized collection logic
5. **Error-Friendly**: Can provide exact source locations for errors across collections
6. **Compatibility**: Existing collections work unchanged
7. **Testable**: Can test Rust logic with mock data, JavaScript logic with mock databases
8. **Namespace Safety**: Automatic collision resolution prevents naming conflicts

## Table ID Collision Examples

### Input Collections
```javascript
// Primary collection
const primary = `
#weapon
1: basic sword
`;

// External collection @rpg/medieval
const medieval = `
#weapon
1: iron sword
2: steel sword

#armor
1: chain mail
`;

// After merging with namespacing:
const merged = `
// External tables get namespaced
#rpg.medieval.weapon
1: iron sword
2: steel sword

#rpg.medieval.armor
1: chain mail

// Primary collection keeps original names
#weapon
1: basic sword
`;
```

### Reference Resolution
```tbl
// User writes this:
#loadout
1: {@rpg/medieval#weapon} and {#weapon}

// Rust transforms external references:
#loadout
1: {#rpg.medieval.weapon} and {#weapon}

// Generates: "steel sword and basic sword"
```

## Considerations

1. **Namespace Clarity**: Use clear namespacing (`publisher.collection.table`) to avoid confusion
2. **Performance**: Large dependency trees resolved in JavaScript could impact performance
3. **Caching Strategy**: JavaScript should cache resolved dependencies aggressively
4. **Error Propagation**: Ensure errors from both JavaScript and Rust sides are clearly distinguished
5. **Version Compatibility**: How to handle different versions of dependencies in the future
6. **Security**: Validate permissions and sandbox execution in JavaScript layer
7. **Memory Usage**: Large merged sources could impact WASM memory limits

## Next Steps

1. **Finalize syntax**: Confirm the `{@username/collection#table}` syntax
2. **Start Phase 1**: Implement AST and parser updates for external references
3. **Create test collections**: Build sample collections to test dependency resolution
4. **Design database schema**: Plan how collections will be stored and versioned
5. **Prototype JavaScript resolver**: Build basic dependency resolution to test the flow
