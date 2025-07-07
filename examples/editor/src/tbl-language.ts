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

    // Register completion provider for TBL language
    monacoInstance.languages.registerCompletionItemProvider("tbl", {
      provideCompletionItems: (model, position) => {
        return provideTblCompletions(model, position, monacoInstance);
      },
    });

    console.log("TBL language registered successfully");
  } catch (error) {
    console.error("Error registering TBL language:", error);
  }
}

// Helper function to provide TBL-specific completions
function provideTblCompletions(
  model: monaco.editor.ITextModel,
  position: monaco.Position,
  monacoInstance: typeof monaco
): monaco.languages.ProviderResult<monaco.languages.CompletionList> {
  const currentLine = model.getLineContent(position.lineNumber);
  const lineUpToCursor = currentLine.substring(0, position.column - 1);

  // Check if we're inside an expression {...}
  if (isInsideExpression(lineUpToCursor)) {
    return provideExpressionCompletions(model, position, monacoInstance);
  }

  // Check if we're inside flags after a table declaration
  if (isInsideTableFlags(lineUpToCursor)) {
    // Check if user has started typing "ex" for "export"
    const flagMatch = lineUpToCursor.match(/\[([a-zA-Z]*)$/);
    const partialFlag = flagMatch ? flagMatch[1] : "";

    if ("export".startsWith(partialFlag.toLowerCase())) {
      return {
        suggestions: [
          {
            label: "export",
            kind: monacoInstance.languages.CompletionItemKind.Keyword,
            documentation:
              "Marks this table as exported and available for external references",
            insertText: "export",
            sortText: "000", // High priority
            range: {
              startLineNumber: position.lineNumber,
              endLineNumber: position.lineNumber,
              startColumn: position.column - partialFlag.length,
              endColumn: position.column,
            },
          },
        ],
      };
    }

    return { suggestions: [] };
  }

  // Check if we're starting a table declaration
  if (isTableDeclarationStart(lineUpToCursor)) {
    const suggestions: monaco.languages.CompletionItem[] = [];

    // If they've started typing #, complete the table name
    const tableMatch = lineUpToCursor.match(/#([a-zA-Z_][a-zA-Z0-9_-]*)?$/);
    if (tableMatch) {
      const partialName = tableMatch[1] || "";
      suggestions.push({
        label: "#table_name",
        kind: monacoInstance.languages.CompletionItemKind.Snippet,
        documentation: "Create a new table with the specified name",
        insertText: "#${1:table_name}",
        insertTextRules:
          monacoInstance.languages.CompletionItemInsertTextRule.InsertAsSnippet,
        sortText: "000", // High priority
        range: {
          startLineNumber: position.lineNumber,
          endLineNumber: position.lineNumber,
          startColumn: position.column - partialName.length - 1, // Include the #
          endColumn: position.column,
        },
      });
    } else {
      // Just starting, suggest the # character
      suggestions.push({
        label: "#table_name",
        kind: monacoInstance.languages.CompletionItemKind.Snippet,
        documentation: "Create a new table with the specified name",
        insertText: "#${1:table_name}",
        insertTextRules:
          monacoInstance.languages.CompletionItemInsertTextRule.InsertAsSnippet,
        sortText: "000", // High priority
        range: {
          startLineNumber: position.lineNumber,
          endLineNumber: position.lineNumber,
          startColumn: position.column,
          endColumn: position.column,
        },
      });
    }

    return { suggestions };
  }

  // Check if we're inside flags after a table declaration
  if (isInsideTableFlags(lineUpToCursor)) {
    return {
      suggestions: [
        {
          label: "export",
          kind: monacoInstance.languages.CompletionItemKind.Keyword,
          documentation:
            "Marks this table as exported and available for external references",
          insertText: "export",
          sortText: "000", // High priority
          range: {
            startLineNumber: position.lineNumber,
            endLineNumber: position.lineNumber,
            startColumn: position.column,
            endColumn: position.column,
          },
        },
      ],
    };
  }

  // Check if we're starting a table declaration
  if (isTableDeclarationStart(lineUpToCursor)) {
    return {
      suggestions: [
        {
          label: "#table_name",
          kind: monacoInstance.languages.CompletionItemKind.Snippet,
          documentation: "Create a new table with the specified name",
          insertText: "#${1:table_name}",
          insertTextRules:
            monacoInstance.languages.CompletionItemInsertTextRule
              .InsertAsSnippet,
          sortText: "000", // High priority
          range: {
            startLineNumber: position.lineNumber,
            endLineNumber: position.lineNumber,
            startColumn: position.column,
            endColumn: position.column,
          },
        },
      ],
    };
  }

  // Check if we're starting a rule (at beginning of line or after whitespace)
  if (isRuleStart(lineUpToCursor)) {
    const suggestions: monaco.languages.CompletionItem[] = [];

    // Check if they've started typing a number
    const numberMatch = lineUpToCursor.match(/(\d*\.?\d*)$/);
    const partialNumber = numberMatch ? numberMatch[1] : "";

    // Common weight suggestions in order of popularity
    const commonWeights = [
      { weight: "1.0", description: "Standard weight (most common)" },
      { weight: "2.0", description: "Double weight (common)" },
      { weight: "0.5", description: "Half weight (uncommon)" },
      { weight: "3.0", description: "Triple weight" },
      { weight: "1.5", description: "Weight 1.5" },
      { weight: "0.25", description: "Quarter weight (rare)" },
    ];

    commonWeights.forEach((item, index) => {
      // Only suggest if it matches what they've started typing
      if (item.weight.startsWith(partialNumber)) {
        suggestions.push({
          label: `${item.weight}: `,
          kind: monacoInstance.languages.CompletionItemKind.Snippet,
          documentation: `Create a rule with ${item.description}`,
          insertText: `${item.weight}: \${1:rule content}`,
          insertTextRules:
            monacoInstance.languages.CompletionItemInsertTextRule
              .InsertAsSnippet,
          sortText: `00${index}`, // Priority based on commonness
          range: {
            startLineNumber: position.lineNumber,
            endLineNumber: position.lineNumber,
            startColumn: position.column - partialNumber.length,
            endColumn: position.column,
          },
        });
      }
    });

    // If no partial number, suggest all weights
    if (!partialNumber) {
      commonWeights.forEach((item, index) => {
        suggestions.push({
          label: `${item.weight}: `,
          kind: monacoInstance.languages.CompletionItemKind.Snippet,
          documentation: `Create a rule with ${item.description}`,
          insertText: `${item.weight}: \${1:rule content}`,
          insertTextRules:
            monacoInstance.languages.CompletionItemInsertTextRule
              .InsertAsSnippet,
          sortText: `00${index}`, // Priority based on commonness
          range: {
            startLineNumber: position.lineNumber,
            endLineNumber: position.lineNumber,
            startColumn: position.column,
            endColumn: position.column,
          },
        });
      });
    }

    return { suggestions };
  }

  return { suggestions: [] };
}

// Helper function to provide completions inside expressions {...}
function provideExpressionCompletions(
  model: monaco.editor.ITextModel,
  position: monaco.Position,
  monacoInstance: typeof monaco
): monaco.languages.CompletionList {
  const suggestions: monaco.languages.CompletionItem[] = [];
  const lineUpToCursor = model
    .getLineContent(position.lineNumber)
    .substring(0, position.column - 1);

  // Extract table names from the entire document (not just text until cursor)
  const fullDocumentText = model.getValue();
  const tableNames = extractTableNames(fullDocumentText);

  // Check what context we're in to prioritize suggestions
  const afterHash = lineUpToCursor.match(/.*{#([a-zA-Z_][a-zA-Z0-9_-]*)?$/);
  const afterPipe = lineUpToCursor.includes("|");
  const justOpenedBrace = lineUpToCursor.endsWith("{");

  console.log({
    afterHash,
    afterPipe,
    justOpenedBrace,
  });
  // Priority 1: Table references if we're after a # character
  if (afterHash) {
    const partialTableName = afterHash[1] || "";
    console.log(tableNames, partialTableName);
    tableNames
      .filter((name) =>
        name.toLowerCase().startsWith(partialTableName.toLowerCase())
      )
      .sort((a, b) => a.toLowerCase().localeCompare(b.toLowerCase()))
      .forEach((tableName, index) => {
        suggestions.push({
          label: `#${tableName}`,
          kind: monacoInstance.languages.CompletionItemKind.Reference,
          documentation: `Reference to table "${tableName}"`,
          insertText: `#${tableName}`,
          sortText: `0${index.toString().padStart(2, "0")}`, // High priority
          range: {
            startLineNumber: position.lineNumber,
            endLineNumber: position.lineNumber,
            startColumn: position.column - partialTableName.length - 1, // Include the # in the range
            endColumn: position.column,
          },
        });
      });

    // Also add remaining table names that don't match the prefix
    tableNames
      .filter(
        (name) => !name.toLowerCase().startsWith(partialTableName.toLowerCase())
      )
      .sort((a, b) => a.toLowerCase().localeCompare(b.toLowerCase()))
      .forEach((tableName, index) => {
        suggestions.push({
          label: `#${tableName}`,
          kind: monacoInstance.languages.CompletionItemKind.Reference,
          documentation: `Reference to table "${tableName}"`,
          insertText: `#${tableName}`,
          sortText: `1${index.toString().padStart(2, "0")}`, // Lower priority
          range: {
            startLineNumber: position.lineNumber,
            endLineNumber: position.lineNumber,
            startColumn: position.column,
            endColumn: position.column,
          },
        });
      });
  }

  // Priority 2: Modifiers if we're after a | character
  if (afterPipe) {
    const modifiers = [
      { label: "capitalize", description: "Capitalize the first letter" },
      { label: "uppercase", description: "Convert to uppercase" },
      { label: "lowercase", description: "Convert to lowercase" },
      { label: "indefinite", description: "Add indefinite article (a/an)" },
      { label: "definite", description: "Add definite article (the)" },
    ];

    modifiers.forEach((modifier, index) => {
      suggestions.push({
        label: modifier.label,
        kind: monacoInstance.languages.CompletionItemKind.Property,
        documentation: modifier.description,
        insertText: modifier.label,
        sortText: `0${index.toString().padStart(2, "0")}`, // High priority for modifiers
        range: {
          startLineNumber: position.lineNumber,
          endLineNumber: position.lineNumber,
          startColumn: position.column,
          endColumn: position.column,
        },
      });
    });
  }

  // Priority 3: Common dice rolls if we just opened a brace or are at the start
  if (justOpenedBrace || (!afterHash && !afterPipe)) {
    // Most common dice rolls first
    const commonDiceRolls = [
      { label: "d6", description: "6-sided die (most common)" },
      { label: "d20", description: "20-sided die (D&D standard)" },
      { label: "2d6", description: "Two 6-sided dice" },
      { label: "d4", description: "4-sided die" },
      { label: "d8", description: "8-sided die" },
      { label: "d10", description: "10-sided die" },
      { label: "d12", description: "12-sided die" },
    ];

    // Less common dice rolls
    const uncommonDiceRolls = [
      { label: "d100", description: "100-sided die (percentile)" },
      { label: "3d6", description: "Three 6-sided dice" },
      { label: "1d4", description: "One 4-sided die" },
    ];

    // Add common dice rolls with high priority
    commonDiceRolls.forEach((dice, index) => {
      const priority = justOpenedBrace
        ? `2${index.toString().padStart(2, "0")}`
        : `3${index.toString().padStart(2, "0")}`;
      suggestions.push({
        label: dice.label,
        kind: monacoInstance.languages.CompletionItemKind.Function,
        documentation: `Dice roll: ${dice.description}`,
        insertText: dice.label,
        sortText: priority,
        range: {
          startLineNumber: position.lineNumber,
          endLineNumber: position.lineNumber,
          startColumn: position.column,
          endColumn: position.column,
        },
      });
    });

    // Add uncommon dice rolls with lower priority
    uncommonDiceRolls.forEach((dice, index) => {
      const priority = justOpenedBrace
        ? `4${index.toString().padStart(2, "0")}`
        : `5${index.toString().padStart(2, "0")}`;
      suggestions.push({
        label: dice.label,
        kind: monacoInstance.languages.CompletionItemKind.Function,
        documentation: `Dice roll: ${dice.description}`,
        insertText: dice.label,
        sortText: priority,
        range: {
          startLineNumber: position.lineNumber,
          endLineNumber: position.lineNumber,
          startColumn: position.column,
          endColumn: position.column,
        },
      });
    });

    // Add table references with lower priority if not after #
    if (!afterHash && tableNames.length > 0) {
      tableNames
        .sort((a, b) => a.toLowerCase().localeCompare(b.toLowerCase()))
        .forEach((tableName, index) => {
          suggestions.push({
            label: `#${tableName}`,
            kind: monacoInstance.languages.CompletionItemKind.Reference,
            documentation: `Reference to table "${tableName}"`,
            insertText: `#${tableName}`,
            sortText: `6${index.toString().padStart(2, "0")}`, // Lower priority when not after #
            range: {
              startLineNumber: position.lineNumber,
              endLineNumber: position.lineNumber,
              startColumn: position.column,
              endColumn: position.column,
            },
          });
        });
    }
  }

  console.log(suggestions);

  return { suggestions };
}

// Helper function to extract table names from the document
function extractTableNames(text: string): string[] {
  const tablePattern = /^#([a-zA-Z_][a-zA-Z0-9_-]*)/gm;
  const matches = text.matchAll(tablePattern);
  const tableNames = new Set<string>();

  for (const match of matches) {
    tableNames.add(match[1]);
  }

  return Array.from(tableNames);
}

// Helper function to check if cursor is inside an expression {...}
function isInsideExpression(lineUpToCursor: string): boolean {
  let braceCount = 0;
  for (const char of lineUpToCursor) {
    if (char === "{") braceCount++;
    if (char === "}") braceCount--;
  }
  return braceCount > 0;
}

// Helper function to check if cursor is inside table flags [...]
function isInsideTableFlags(lineUpToCursor: string): boolean {
  // Look for pattern: #tablename[... with unclosed bracket
  const tableWithFlagsPattern = /#[a-zA-Z_][a-zA-Z0-9_-]*\[/;
  const match = lineUpToCursor.match(tableWithFlagsPattern);

  if (!match) return false;

  // Check if bracket is still open (no closing ] after the match)
  const afterMatch = lineUpToCursor.substring(match.index! + match[0].length);
  return !afterMatch.includes("]");
}

// Helper function to check if we're starting a table declaration
function isTableDeclarationStart(lineUpToCursor: string): boolean {
  // Check if line starts with optional whitespace followed by # or just whitespace
  return /^\s*#?$/.test(lineUpToCursor);
}

// Helper function to check if we're starting a rule
function isRuleStart(lineUpToCursor: string): boolean {
  // Check if line is empty or contains only whitespace, and we're not in a table declaration
  return /^\s*$/.test(lineUpToCursor) && !lineUpToCursor.includes("#");
}
