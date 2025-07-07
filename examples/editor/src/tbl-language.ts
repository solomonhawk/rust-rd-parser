import * as monaco from "monaco-editor";

// Define the TBL language configuration for Monaco Editor
export const tblLanguageConfig: monaco.languages.LanguageConfiguration = {
  comments: {
    lineComment: "//",
    blockComment: ["/*", "*/"],
  },
  brackets: [
    ["{", "}"],
    ["[", "]"],
  ],
  autoClosingPairs: [
    { open: "{", close: "}" },
    { open: "[", close: "]" },
    { open: '"', close: '"' },
    { open: "'", close: "'" },
  ],
  surroundingPairs: [
    { open: "{", close: "}" },
    { open: "[", close: "]" },
    { open: '"', close: '"' },
    { open: "'", close: "'" },
  ],
  folding: {
    markers: {
      start: /^\s*#\w+/,
      end: /^\s*$/,
    },
  },
};

// Define the TBL language tokens for syntax highlighting
export const tblLanguageTokens: monaco.languages.IMonarchLanguage = {
  // Set defaultToken to invalid to see what you do not tokenize yet
  defaultToken: "invalid",
  ignoreCase: false,

  // Define keywords and operators
  keywords: ["export"],
  operators: [":", "#", "d"],

  // Define symbols for tokenization
  symbols: /[=><!~?:&|+\-*/ ^%]+/,
  escapes:
    /\\(?:[abfnrtv\\"']|x[0-9A-Fa-f]{1,4}|u[0-9A-Fa-f]{4}|U[0-9A-Fa-f]{8})/,
  digits: /\d+(\.\d+)?/,

  // The main tokenizer
  tokenizer: {
    root: [
      // Line comments
      [/\/\/.*$/, "comment"],

      // Block comments
      [/\/\*/, "comment", "@comment"],

      // Table declarations - simplified
      [
        /^(\s*)(#)([a-zA-Z_][a-zA-Z0-9_-]*)/,
        ["white", "keyword.table", "entity.name.table"],
      ],

      // Table flags
      [/\[/, "delimiter.bracket", "@flags"],

      // Rule weights (numbers followed by colon)
      [
        /^(\s*)([0-9]*\.?[0-9]+)(\s*)(:)/,
        ["white", "number", "white", "delimiter"],
      ],

      // Dice roll expressions - simplified
      [/\{[0-9]*d[0-9]+\}/, "keyword.dice"],

      // Table reference expressions - simplified (with modifiers)
      [/\{#[a-zA-Z_][a-zA-Z0-9_-]*(\|[a-zA-Z]+)*\}/, "variable.table"],

      // Generic expressions (fallback)
      [/\{/, "delimiter", "@expression"],

      // Numbers
      [/[0-9]+(\.[0-9]+)?/, "number"],

      // Strings (rule text)
      [/[^{}\n\r#]+/, "string"],

      // Whitespace
      [/\s+/, "white"],
    ],

    comment: [
      [/[^/*]+/, "comment"],
      [/\*\//, "comment", "@pop"],
      [/[/*]/, "comment"],
    ],

    flags: [
      [/export/, "keyword"],
      [/[a-zA-Z_][a-zA-Z0-9_]*/, "identifier"],
      [/\]/, "delimiter.bracket", "@pop"],
      [/\s+/, "white"],
    ],

    expression: [
      // Dice rolls inside expressions
      [/([0-9]+)?(d)([0-9]+)/, "keyword.dice"],

      // Table references inside expressions with optional modifiers
      [/(#)([a-zA-Z_][a-zA-Z0-9_-]*)/, "variable.table"],

      // Modifier separators and keywords
      [/\|/, "delimiter"],
      [/(indefinite|definite|capitalize|uppercase|lowercase)/, "keyword"],

      [/[^}]+/, "string"],
      [/\}/, "delimiter", "@pop"],
    ],
  },
};

// Define theme colors for TBL language
export const tblTheme: monaco.editor.IStandaloneThemeData = {
  base: "vs-dark",
  inherit: true,
  rules: [
    { token: "comment", foreground: "6A9955", fontStyle: "italic" },
    { token: "keyword.table", foreground: "C586C0" },
    { token: "entity.name.table", foreground: "DCDCAA" },
    { token: "keyword", foreground: "C586C0" },
    { token: "number.weight", foreground: "B5CEA8" },
    { token: "number.dice", foreground: "B5CEA8" },
    { token: "keyword.dice", foreground: "C586C0", fontStyle: "bold" },
    { token: "keyword.reference", foreground: "C586C0" },
    { token: "variable.table", foreground: "9CDCFE" },
    { token: "delimiter.colon", foreground: "D4D4D4" },
    { token: "delimiter.curly", foreground: "FFD700", fontStyle: "bold" },
    { token: "bracket.square", foreground: "D4D4D4" },
    { token: "string", foreground: "CE9178" },
  ],
  colors: {},
};

// Function to register the TBL language with Monaco
export function registerTblLanguage(monacoInstance: typeof monaco) {
  try {
    // Check if language is already registered
    const languages = monacoInstance.languages.getLanguages();

    if (languages.some((lang) => lang.id === "tbl")) {
      console.log("TBL language already registered");
      return;
    }

    monacoInstance.languages.register({ id: "tbl" });
    monacoInstance.languages.setLanguageConfiguration("tbl", tblLanguageConfig);
    monacoInstance.languages.setMonarchTokensProvider("tbl", tblLanguageTokens);
    monacoInstance.editor.defineTheme("tbl-dark", tblTheme);

    console.log("TBL language registered successfully");
  } catch (error) {
    console.error("Error registering TBL language:", error);
  }
}
