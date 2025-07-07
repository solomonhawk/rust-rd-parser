#!/usr/bin/env node

// Simple test script to verify WASM functionality
const init = require("./pkg/parser.js");

async function testWasm() {
  try {
    // Initialize the WASM module
    await init();
    const {
      WasmParser,
      WasmCollection,
      WasmUtils,
    } = require("./pkg/parser.js");

    console.log("🧪 Testing TBL Parser WASM Module\n");

    // Test 1: Parse simple TBL
    console.log("Test 1: Parse TBL source");
    const source = `#color
1.0: red
2.0: blue

#shape
1.0: circle
2.0: square

#item
1.0: {#color} {#shape}`;

    try {
      const parseResult = WasmParser.parse(source);
      console.log("✅ Parsing successful");
      const ast = JSON.parse(parseResult);
      console.log(`   Tables found: ${ast.tables.length}`);
    } catch (error) {
      console.log(`❌ Parsing failed: ${error}`);
    }

    // Test 2: Generate content
    console.log("\nTest 2: Generate content");
    try {
      const collection = new WasmCollection(source);
      console.log("✅ Collection created successfully");

      // Test table existence
      console.log(`   Has 'color' table: ${collection.has_table("color")}`);
      console.log(
        `   Has 'nonexistent' table: ${collection.has_table("nonexistent")}`
      );

      // Test table ID listing
      const tableIds = collection.get_table_ids();
      console.log(`   Table IDs: ${tableIds.join(", ")}`);

      // Generate some content
      const generated = collection.generate("item", 3);
      console.log(`   Generated items: ${generated}`);
    } catch (error) {
      console.log(`❌ Collection test failed: ${error}`);
    }

    // Test 3: Error handling
    console.log("\nTest 3: Error handling");
    try {
      const badSource = "invalid tbl syntax";
      const parseResult = WasmParser.parse(badSource);
      console.log("❌ Should have failed parsing invalid syntax");
    } catch (error) {
      console.log("✅ Correctly caught parse error");
    }

    console.log("\n🎉 WASM tests completed!");
  } catch (error) {
    console.error("❌ WASM initialization failed:", error);
  }
}

testWasm();
