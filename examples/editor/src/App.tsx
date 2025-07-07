import Editor, { useMonaco } from "@monaco-editor/react";
import * as monaco from "monaco-editor";
import { useEffect, useRef, useState, useCallback } from "react";
import "./App.css";
import { registerTblLanguage } from "./tbl-language";
import { useTblWorker, type TblDiagnostic } from "./useTblWorker";

function App() {
  const monacoInstance = useMonaco();
  const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  const [isLanguageReady, setIsLanguageReady] = useState(false);
  const [diagnostics, setDiagnostics] = useState<TblDiagnostic[]>([]);
  const [isValidating, setIsValidating] = useState(false);
  const [tableIds, setTableIds] = useState<string[]>([]);
  const [generatedContent, setGeneratedContent] = useState<string>("");
  const [isGenerating, setIsGenerating] = useState(false);
  const [showOnlyExported, setShowOnlyExported] = useState(false);

  // Initialize TBL worker
  const { validateSyntax, getTableIds, getExportedTableIds, generateContent } =
    useTblWorker();

  // Simple worker ready check - if functions are available, worker is ready
  const isWorkerReady =
    !!validateSyntax &&
    !!getTableIds &&
    !!getExportedTableIds &&
    !!generateContent;

  const [code, setCode] = useState(`// Welcome to the TBL Language Editor!
// This editor supports the TBL (Table) format with dice roll expressions and comments

// Example table with dice rolls:
#weapons
1.0: Sword (damage: {2d6} + {1d4})
2.0: Bow (damage: {1d8})
1.5: Dagger (damage: {1d4})

// Table with references:
#armor[export]
1.0: {#material} armor
2.0: {#material} shield

#material
1.0: leather
2.0: iron
1.5: steel

// Test error handling - try adding syntax errors below:
// Uncomment the lines below to test error diagnostics:
// #invalid_table
// invalid_weight: item
// missing_colon item
`);

  // Register the TBL language when Monaco is available
  useEffect(() => {
    if (monacoInstance) {
      registerTblLanguage(monacoInstance);
      setIsLanguageReady(true);
    }
  }, [monacoInstance]);

  // Validate syntax and update markers, also fetch table IDs
  const validateAndUpdateMarkers = useCallback(
    async (content: string) => {
      if (!isWorkerReady || !editorRef.current || !monacoInstance) {
        return;
      }

      setIsValidating(true);
      try {
        // Validate syntax
        const result = await validateSyntax(content);
        setDiagnostics(result.diagnostics);

        // If validation is successful, fetch table IDs
        if (result.isValid) {
          try {
            const [allTablesResult, exportedTablesResult] = await Promise.all([
              getTableIds(content),
              getExportedTableIds(content),
            ]);

            if (allTablesResult.success && allTablesResult.tableIds) {
              // Store all table IDs and filter based on user preference
              const tablesToShow =
                showOnlyExported &&
                exportedTablesResult.success &&
                exportedTablesResult.tableIds
                  ? exportedTablesResult.tableIds
                  : allTablesResult.tableIds;
              setTableIds(tablesToShow);
            }
          } catch (error) {
            console.error("Failed to get table IDs:", error);
            setTableIds([]);
          }
        } else {
          setTableIds([]);
        }

        // Convert diagnostics to Monaco markers
        const markers: monaco.editor.IMarkerData[] = result.diagnostics.map(
          (diagnostic) => ({
            message: diagnostic.message,
            severity:
              diagnostic.severity === "error"
                ? monaco.MarkerSeverity.Error
                : diagnostic.severity === "warning"
                ? monaco.MarkerSeverity.Warning
                : monaco.MarkerSeverity.Info,
            startLineNumber: diagnostic.line,
            startColumn: diagnostic.column,
            endLineNumber: diagnostic.endLine,
            endColumn: diagnostic.endColumn,
          })
        );

        // Set markers on the editor model
        const model = editorRef.current.getModel();
        if (model) {
          monacoInstance.editor.setModelMarkers(model, "tbl", markers);
        }
      } catch (error) {
        console.error("Validation error:", error);
        setDiagnostics([
          {
            message: `Validation failed: ${
              error instanceof Error ? error.message : String(error)
            }`,
            severity: "error",
            line: 0,
            column: 0,
            endLine: 0,
            endColumn: 0,
          },
        ]);
        setTableIds([]);
      } finally {
        setIsValidating(false);
      }
    },
    [
      isWorkerReady,
      monacoInstance,
      validateSyntax,
      getTableIds,
      getExportedTableIds,
      showOnlyExported,
    ]
  );

  // Debounced validation
  useEffect(() => {
    const timeoutId = setTimeout(() => {
      validateAndUpdateMarkers(code);
    }, 500); // 500ms debounce

    return () => clearTimeout(timeoutId);
  }, [code, validateAndUpdateMarkers]);

  // Re-validate when filter changes to update table list
  useEffect(() => {
    if (isWorkerReady && code) {
      validateAndUpdateMarkers(code);
    }
  }, [showOnlyExported, isWorkerReady, code, validateAndUpdateMarkers]);

  // Handle table generation
  const handleGenerateFromTable = useCallback(
    async (tableId: string, count: number = 1) => {
      if (!isWorkerReady) {
        return;
      }

      setIsGenerating(true);
      try {
        const result = await generateContent(code, tableId, count);
        if (result.success && result.generated) {
          setGeneratedContent((prev) => {
            const timestamp = new Date().toLocaleTimeString();
            const newContent = `[${timestamp}] Generated from "${tableId}" (x${count}):\n${result.generated}\n\n`;
            return newContent + prev;
          });
        } else {
          setGeneratedContent((prev) => {
            const timestamp = new Date().toLocaleTimeString();
            const errorContent = `[${timestamp}] Error generating from "${tableId}": ${
              result.error || "Unknown error"
            }\n\n`;
            return errorContent + prev;
          });
        }
      } catch (error) {
        console.error("Generation error:", error);
        setGeneratedContent((prev) => {
          const timestamp = new Date().toLocaleTimeString();
          const errorContent = `[${timestamp}] Generation failed: ${
            error instanceof Error ? error.message : String(error)
          }\n\n`;
          return errorContent + prev;
        });
      } finally {
        setIsGenerating(false);
      }
    },
    [isWorkerReady, code, generateContent]
  );

  const handleEditorChange = (value: string | undefined) => {
    setCode(value || "");
  };

  const handleEditorDidMount = (
    editor: monaco.editor.IStandaloneCodeEditor,
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    _monacoApi: typeof monaco
  ) => {
    editorRef.current = editor;
  };

  return (
    <div style={{ height: "100vh", display: "flex", flexDirection: "column" }}>
      <header style={{ padding: "1rem", borderBottom: "1px solid #ccc" }}>
        <h1>TBL Language Editor</h1>
        <p>
          A web editor for table-based random generation with dice roll
          expressions
        </p>

        {/* Worker Status */}
        <div style={{ marginTop: "0.5rem", fontSize: "0.9rem" }}>
          {!isWorkerReady && (
            <span style={{ color: "#ffa500" }}>🔄 Initializing...</span>
          )}
          {isWorkerReady && <span style={{ color: "#44aa44" }}>✅ Ready</span>}
          {isValidating && (
            <span style={{ color: "#4444ff", marginLeft: "1rem" }}>
              🔍 Validating...
            </span>
          )}
          {isGenerating && (
            <span style={{ color: "#9944ff", marginLeft: "1rem" }}>
              🎲 Generating...
            </span>
          )}
        </div>

        {/* Diagnostics Summary */}
        {diagnostics.length > 0 && (
          <div style={{ marginTop: "0.5rem", fontSize: "0.9rem" }}>
            <span style={{ color: "#ff4444" }}>
              ⚠️ {diagnostics.length} issue{diagnostics.length > 1 ? "s" : ""}{" "}
              found
            </span>
          </div>
        )}

        {/* Table Summary */}
        {tableIds.length > 0 && (
          <div style={{ marginTop: "0.5rem", fontSize: "0.9rem" }}>
            <span style={{ color: "#4444aa" }}>
              📋 {tableIds.length} table{tableIds.length > 1 ? "s" : ""} found:{" "}
              {tableIds.join(", ")}
            </span>
          </div>
        )}
      </header>

      <div style={{ flex: 1, display: "flex" }}>
        {/* Editor Panel */}
        <div
          style={{ flex: 1, padding: "1rem", borderRight: "1px solid #ccc" }}
        >
          <Editor
            height="100%"
            language={isLanguageReady ? "tbl" : "plaintext"}
            value={code}
            onChange={handleEditorChange}
            onMount={handleEditorDidMount}
            theme={isLanguageReady ? "tbl-dark" : "vs-dark"}
            options={{
              minimap: { enabled: false },
              fontSize: 14,
              lineNumbers: "on",
              roundedSelection: false,
              scrollBeyondLastLine: false,
              automaticLayout: true,
              wordWrap: "on",
            }}
          />
        </div>

        {/* Table Generation Panel */}
        <div
          style={{
            width: "320px",
            padding: "1rem",
            display: "flex",
            flexDirection: "column",
            backgroundColor: "#222",
            color: "white",
            borderLeft: "1px solid #dee2e6",
          }}
        >
          <h3 style={{ margin: "0 0 1rem 0", fontSize: "1.1rem" }}>
            Table Generation
          </h3>

          {/* Filter Toggle */}
          <div
            style={{
              marginBottom: "1rem",
              padding: "0.5rem",
              backgroundColor: "#333",
              borderRadius: "4px",
              border: "1px solid #555",
            }}
          >
            <label
              style={{
                display: "flex",
                alignItems: "center",
                cursor: "pointer",
                fontSize: "0.9rem",
              }}
            >
              <input
                type="checkbox"
                checked={showOnlyExported}
                onChange={(e) => setShowOnlyExported(e.target.checked)}
                style={{ marginRight: "0.5rem" }}
              />
              Show only exported tables
            </label>
            <div
              style={{
                fontSize: "0.8rem",
                color: "#aaa",
                marginTop: "0.25rem",
                marginLeft: "1.5rem",
              }}
            >
              {showOnlyExported
                ? "Showing tables marked with [export]"
                : "Showing all tables"}
            </div>
          </div>

          {!isWorkerReady ? (
            <div style={{ color: "#666", fontStyle: "italic" }}>
              Waiting for worker to initialize...
            </div>
          ) : tableIds.length === 0 ? (
            <div style={{ color: "#666", fontStyle: "italic" }}>
              No valid tables found. Create a table with the #tablename syntax.
            </div>
          ) : (
            <div
              style={{
                display: "flex",
                flexDirection: "column",
                gap: "0.5rem",
              }}
            >
              <div style={{ fontWeight: "bold", marginBottom: "0.5rem" }}>
                Available Tables:
              </div>
              {tableIds.map((tableId) => (
                <div
                  key={tableId}
                  style={{
                    display: "flex",
                    gap: "0.5rem",
                    alignItems: "center",
                    padding: "0.5rem",
                    borderRadius: "4px",
                    border: "1px solid #dee2e6",
                  }}
                >
                  <span
                    style={{
                      flex: 1,
                      fontFamily: "monospace",
                      fontSize: "0.9rem",
                    }}
                  >
                    #{tableId}
                  </span>
                  <button
                    onClick={() => handleGenerateFromTable(tableId, 1)}
                    disabled={isGenerating}
                    style={{
                      padding: "0.25rem 0.5rem",
                      fontSize: "0.8rem",
                      backgroundColor: "#007bff",
                      color: "white",
                      border: "none",
                      borderRadius: "3px",
                      cursor: isGenerating ? "not-allowed" : "pointer",
                      opacity: isGenerating ? 0.6 : 1,
                    }}
                  >
                    Generate x1
                  </button>
                  <button
                    onClick={() => handleGenerateFromTable(tableId, 5)}
                    disabled={isGenerating}
                    style={{
                      padding: "0.25rem 0.5rem",
                      fontSize: "0.8rem",
                      backgroundColor: "#28a745",
                      color: "white",
                      border: "none",
                      borderRadius: "3px",
                      cursor: isGenerating ? "not-allowed" : "pointer",
                      opacity: isGenerating ? 0.6 : 1,
                    }}
                  >
                    Generate x5
                  </button>
                </div>
              ))}
            </div>
          )}

          {/* Generated Content Display */}
          <div
            style={{
              marginTop: "1.5rem",
              flex: 1,
              display: "flex",
              flexDirection: "column",
            }}
          >
            <div
              style={{
                display: "flex",
                justifyContent: "space-between",
                alignItems: "center",
                marginBottom: "0.5rem",
              }}
            >
              <h4 style={{ margin: 0, fontSize: "1rem" }}>
                Generated Results:
              </h4>
              {generatedContent && (
                <button
                  onClick={() => setGeneratedContent("")}
                  style={{
                    padding: "0.25rem 0.5rem",
                    fontSize: "0.8rem",
                    backgroundColor: "#dc3545",
                    color: "white",
                    border: "none",
                    borderRadius: "3px",
                    cursor: "pointer",
                  }}
                >
                  Clear
                </button>
              )}
            </div>
            <div
              style={{
                flex: 1,

                border: "1px solid #dee2e6",
                borderRadius: "4px",
                padding: "0.75rem",
                fontFamily: "monospace",
                fontSize: "0.85rem",
                whiteSpace: "pre-wrap",
                overflow: "auto",
                minHeight: "150px",
                maxHeight: "300px",
                backgroundColor: "#222",
                color: "white",
              }}
            >
              {generatedContent ||
                "No generated content yet. Click a generate button above."}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
