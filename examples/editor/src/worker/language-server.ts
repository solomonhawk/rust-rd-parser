import type * as monaco from "monaco-editor";

// Helper function to provide TBL-specific completions
export function provideTblCompletions(
  lineUpToCursor: string,
  fullText: string,
  position: {
    lineNumber: number;
    column: number;
  }
) {
  // Check if we're inside an expression {...}
  if (isInsideExpression(lineUpToCursor)) {
    return provideExpressionCompletions(lineUpToCursor, fullText, position);
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
            kind: 17,
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
        kind: 27,
        documentation: "Create a new table with the specified name",
        insertText: "#${1:table_name}",
        insertTextRules: 4,
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
        kind: 27,
        documentation: "Create a new table with the specified name",
        insertText: "#${1:table_name}",
        insertTextRules: 4,
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
          kind: 17,
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
          kind: 27,
          documentation: "Create a new table with the specified name",
          insertText: "#${1:table_name}",
          insertTextRules: 4,
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
          kind: 27,
          documentation: `Create a rule with ${item.description}`,
          insertText: `${item.weight}: \${1:rule content}`,
          insertTextRules: 4,
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
          kind: 27,
          documentation: `Create a rule with ${item.description}`,
          insertText: `${item.weight}: \${1:rule content}`,
          insertTextRules: 4,
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
  lineUpToCursor: string,
  fullText: string,
  position: { lineNumber: number; column: number }
) {
  const suggestions: monaco.languages.CompletionItem[] = [];

  // Extract table names from the entire document (not just text until cursor)
  const tableNames = extractTableNames(fullText);

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
          kind: 21,
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
          kind: 21,
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
        kind: 9,
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
        kind: 1,
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
        kind: 1,
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
            kind: 21,
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
