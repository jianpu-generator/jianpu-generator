export type {
  DiagnosticMessageOut as DiagnosticMessage,
  DiagnosticOut as Diagnostic,
  DiagnosticViewZoneOut as DiagnosticViewZone,
  GeneratePdfResponse as GeneratePdfResult,
  GenerateSplitPdfsResponse as GenerateSplitPdfResult,
  GenerateWavResponse as GenerateWavResult,
  ListMeasureSpansResponse as ListMeasureSpansResult,
  ListPartsResponse as ListPartsResult,
  ListScoreLineHintsResponse as ListScoreLineHintsResult,
  MeasureAtOffsetResponse as MeasureAtOffsetResult,
  MeasureSpanOut as MeasureSpan,
  PartOut as PartInfo,
  RenderResponse as RenderResult,
  ScoreLineHintOut as ScoreLineHint,
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
  /** Move the cursor to the given JS string char offset and reveal the line. */
  jumpToOffset: (charOffset: number) => void
  focus: () => void
  getEditor: () => import('monaco-editor').editor.IStandaloneCodeEditor | null
}
