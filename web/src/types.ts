export type {
  DiagnosticMessageOut as DiagnosticMessage,
  DiagnosticOut as Diagnostic,
  DiagnosticViewZoneOut as DiagnosticViewZone,
  GeneratePdfResponse as GeneratePdfResult,
  GenerateSplitPdfsResponse as GenerateSplitPdfResult,
  GenerateWavResponse as GenerateWavResult,
  ListMeasureSpansResponse as ListMeasureSpansResult,
  ListPartsResponse as ListPartsResult,
  MeasureAtOffsetResponse as MeasureAtOffsetResult,
  MeasureSpanOut as MeasureSpan,
  PartOut as PartInfo,
  RenderResponse as RenderResult,
  SpanOut as ByteSpan,
} from 'jianpu-wasm'

interface EditorSelection {
  start: number
  end: number
}

export interface EditorHandle {
  /** Insert text at the current cursor, replacing any selection. */
  insertAtCursor: (text: string) => void
  getSelection: () => EditorSelection
  setSelection: (start: number, end: number) => void
  /** Select a range of lines by 1-indexed line numbers and reveal the start. */
  setSelectionByLines: (startLine: number, endLine: number) => void
  /** Move the cursor to the given JS string char offset and reveal the line. */
  jumpToOffset: (charOffset: number) => void
  focus: () => void
  getEditor: () => import('monaco-editor').editor.IStandaloneCodeEditor | null
}
