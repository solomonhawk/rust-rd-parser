/// <reference lib="webworker" />

import init, {
  WasmParser,
  WasmUtils,
  WasmCollection,
} from "../../public/wasm/table_collection.js";
import { provideTblCompletions } from "./language-server";

// eslint-disable-next-line @typescript-eslint/no-unused-vars
declare const self: DedicatedWorkerGlobalScope;

interface WasmDiagnostic {
  message: string;
  severity: string;
  line: number;
  column: number;
  end_line: number;
  end_column: number;
  source?: string;
}

interface WasmParseResult {
  success: boolean;
  ast_json?: string;
  diagnostics: WasmDiagnostic[];
}

export async function parseWithDiagnostics(content: string): Promise<{
  success: boolean;
  astJson?: string;
  diagnostics: Array<{
    message: string;
    severity: "error" | "warning" | "info";
    line: number;
    column: number;
    endLine: number;
    endColumn: number;
    source: string;
  }>;
}> {
  await init();

  try {
    // Call the WASM function for parsing with diagnostics
    const resultJson = WasmParser.parse_with_diagnostics(content);
    const result: WasmParseResult = JSON.parse(resultJson);

    return {
      success: result.success,
      astJson: result.ast_json,
      diagnostics: result.diagnostics.map((d: WasmDiagnostic) => ({
        message: d.message,
        severity: (d.severity as "error" | "warning" | "info") || "error",
        line: d.line,
        column: d.column,
        endLine: d.end_line,
        endColumn: d.end_column,
        source: d.source || "",
      })),
    };
  } catch (error) {
    console.error("WASM parsing error:", error);
    return {
      success: false,
      diagnostics: [
        {
          message: `WASM Error: ${
            error instanceof Error ? error.message : String(error)
          }`,
          severity: "error",
          line: 0,
          column: 0,
          endLine: 0,
          endColumn: 0,
          source: "",
        },
      ],
    };
  }
}

export async function validateSyntax(content: string): Promise<{
  isValid: boolean;
  diagnostics: Array<{
    message: string;
    severity: "error" | "warning" | "info";
    line: number;
    column: number;
    endLine: number;
    endColumn: number;
  }>;
}> {
  await init();

  try {
    const resultJson = WasmParser.validate_with_diagnostics(content);
    const result: WasmParseResult = JSON.parse(resultJson);

    return {
      isValid: result.success,
      diagnostics: result.diagnostics.map((d: WasmDiagnostic) => ({
        message: d.message,
        severity: (d.severity as "error" | "warning" | "info") || "error",
        line: d.line,
        column: d.column,
        endLine: d.end_line,
        endColumn: d.end_column,
      })),
    };
  } catch (error) {
    console.error("WASM validation error:", error);
    return {
      isValid: false,
      diagnostics: [
        {
          message: `WASM Error: ${
            error instanceof Error ? error.message : String(error)
          }`,
          severity: "error",
          line: 0,
          column: 0,
          endLine: 0,
          endColumn: 0,
        },
      ],
    };
  }
}

export async function generateContent(
  content: string,
  tableId: string,
  count: number = 1
): Promise<{
  success: boolean;
  generated?: string;
  error?: string;
}> {
  await init();

  try {
    // Create a collection from the content
    const collection = new WasmCollection(content);
    const result = collection.generate(tableId, count);
    collection.free(); // Clean up WASM memory

    return {
      success: true,
      generated: result,
    };
  } catch (error) {
    console.error("WASM generation error:", error);
    return {
      success: false,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

export async function getTableIds(content: string): Promise<{
  success: boolean;
  tableIds?: string[];
  error?: string;
}> {
  await init();

  try {
    const collection = new WasmCollection(content);
    const tableIds = collection.get_table_ids();
    collection.free(); // Clean up WASM memory

    return {
      success: true,
      tableIds,
    };
  } catch (error) {
    console.error("WASM table ID extraction error:", error);
    return {
      success: false,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

export async function getExportedTableIds(content: string): Promise<{
  success: boolean;
  tableIds?: string[];
  error?: string;
}> {
  await init();

  try {
    const collection = new WasmCollection(content);
    const tableIds = collection.get_exported_table_ids();
    collection.free(); // Clean up WASM memory

    return {
      success: true,
      tableIds,
    };
  } catch (error) {
    console.error("WASM exported table ID extraction error:", error);
    return {
      success: false,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

export async function getVersion(): Promise<string> {
  await init();
  return WasmUtils.version();
}

export async function getExampleSource(): Promise<string> {
  await init();
  return WasmUtils.example_source();
}

export async function getCompletions(
  lineUpToCursor: string,
  fullText: string,
  position: {
    lineNumber: number;
    column: number;
  }
) {
  return provideTblCompletions(lineUpToCursor, fullText, position);
}
