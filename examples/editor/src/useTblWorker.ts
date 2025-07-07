import { useCallback } from "react";
import { workerInstance } from "./worker";

// Type definitions for better TypeScript support
export interface TblDiagnostic {
  message: string;
  severity: "error" | "warning" | "info";
  line: number;
  column: number;
  endLine: number;
  endColumn: number;
  source?: string;
}

export interface ParseResult {
  success: boolean;
  astJson?: string;
  diagnostics: TblDiagnostic[];
}

export interface ValidationResult {
  isValid: boolean;
  diagnostics: TblDiagnostic[];
}

export interface GenerationResult {
  success: boolean;
  generated?: string;
  error?: string;
}

export interface TableIdsResult {
  success: boolean;
  tableIds?: string[];
  error?: string;
}

export function useTblWorker() {
  const parseWithDiagnostics = useCallback(
    async (content: string): Promise<ParseResult> => {
      return workerInstance.parseWithDiagnostics(content);
    },
    []
  );

  const validateSyntax = useCallback(
    async (content: string): Promise<ValidationResult> => {
      return workerInstance.validateSyntax(content);
    },
    []
  );

  const generateContent = useCallback(
    async (
      content: string,
      tableId: string,
      count: number = 1
    ): Promise<GenerationResult> => {
      return workerInstance.generateContent(content, tableId, count);
    },
    []
  );

  const getTableIds = useCallback(
    async (content: string): Promise<TableIdsResult> => {
      return workerInstance.getTableIds(content);
    },
    []
  );

  const getVersion = useCallback(async (): Promise<string> => {
    return workerInstance.getVersion();
  }, []);

  const getExampleSource = useCallback(async (): Promise<string> => {
    return workerInstance.getExampleSource();
  }, []);

  return {
    parseWithDiagnostics,
    validateSyntax,
    generateContent,
    getTableIds,
    getVersion,
    getExampleSource,
  };
}
