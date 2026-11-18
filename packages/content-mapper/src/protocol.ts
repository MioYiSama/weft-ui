// The subset of the TypeScript content mapper protocol this mapper uses:
// https://github.com/microsoft/typescript-go/pull/4712
import type { PositionEncoding, SpanMapping } from "@weft-ui/compiler";

export type { PositionEncoding, SpanMapping };

export interface InitializeParams {
  /** Encodings TypeScript supports; the mapper picks one. */
  positionEncodings: PositionEncoding[];
  locale?: string;
}

export interface InitializeResult {
  positionEncoding: PositionEncoding;
  /** Shown with mapper diagnostics. */
  diagnosticSource: string;
}

export interface OpenProjectParams {
  configFileName: string;
  projectHandle: string;
  options?: Record<string, unknown>;
  compilerOptions: Record<string, unknown>;
}

export interface TransformParams {
  fileName: string;
  /** Original content of the file. */
  content: string;
  projectHandle: string;
}

export interface MapperDiagnostic {
  messageText: string;
  start: number;
  length: number;
  code: number;
}

export interface TransformResult {
  text: string;
  extension: ".ts";
  mappings: SpanMapping[];
  diagnostics: MapperDiagnostic[];
}

export interface CloseProjectParams {
  projectHandle: string;
}
