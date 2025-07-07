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

  // Initialize TBL worker
  const { validateSyntax } = useTblWorker();

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

  // Validate syntax and update markers
  const validateAndUpdateMarkers = useCallback(
    async (content: string) => {
      if (!editorRef.current || !monacoInstance) {
        return;
      }

      setIsValidating(true);
      try {
        const result = await validateSyntax(content);
        setDiagnostics(result.diagnostics);

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
            startLineNumber: diagnostic.line, // Monaco is 1-based
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
      } finally {
        setIsValidating(false);
      }
    },
    [monacoInstance, validateSyntax]
  );

  // Debounced validation
  useEffect(() => {
    const timeoutId = setTimeout(() => {
      validateAndUpdateMarkers(code);
    }, 500); // 500ms debounce

    return () => clearTimeout(timeoutId);
  }, [code, validateAndUpdateMarkers]);

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
          {isValidating && (
            <span style={{ color: "#4444ff", marginLeft: "1rem" }}>
              🔍 Validating...
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
      </header>

      <div style={{ flex: 1, padding: "1rem" }}>
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
    </div>
  );
}

export default App;
