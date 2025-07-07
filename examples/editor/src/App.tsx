import Editor, { useMonaco } from "@monaco-editor/react";
import * as monaco from "monaco-editor";
import { useEffect, useRef, useState } from "react";
import "./App.css";
import { registerTblLanguage } from "./tbl-language";

function App() {
  const monacoInstance = useMonaco();
  const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  const [isLanguageReady, setIsLanguageReady] = useState(false);
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
1.5: steel`);

  // Register the TBL language when Monaco is available
  useEffect(() => {
    if (monacoInstance) {
      registerTblLanguage(monacoInstance);
      setIsLanguageReady(true);
    }
  }, [monacoInstance]);

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
        <div style={{ fontSize: "12px", color: "#666" }}>
          Debug: Monaco loaded: {monacoInstance ? "✓" : "✗"} | Language ready:{" "}
          {isLanguageReady ? "✓" : "✗"} | Current language:{" "}
          {isLanguageReady ? "tbl" : "plaintext"} | Current theme:{" "}
          {isLanguageReady ? "tbl-dark" : "vs-dark"}
        </div>
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
