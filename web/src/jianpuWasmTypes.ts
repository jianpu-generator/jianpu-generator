// WIT-boundary output shapes, matching the old `pkg/jianpu_wasm.d.ts`
// (`tsify`-generated) field names/shapes exactly, so consumers elsewhere in
// `web/` keep working unchanged after the wit-bindgen/jco cutover — split
// out of `jianpuWasm.ts` purely to stay under the 400-line-per-file cap.
import type {
  LyricSelectionRun as LyricSelectionRunOut,
  LyricSpan as LyricSpanOut,
  NoteSelectionRun as NoteSelectionRunOut,
  NoteSpan as NoteSpanOut,
  PartDeclaration as PartDeclarationOut,
  Span as SpanOut,
  TextEdit as TextEditOut,
  Diagnostic as WitDiagnostic,
} from '../../crates/jianpu-wasm/pkg-component/jianpu_wasm.js'

// ---- pass-through type aliases (field names/shapes already identical
// between the old tsify output and the new wit-bindgen/jco output) ----

export type { SpanOut }
export type DiagnosticOut = WitDiagnostic
export interface DiagnosticMessageOut {
  message: string
}
export type NoteSpan = NoteSpanOut
export type {
  LyricSelectionRunOut,
  LyricSpanOut,
  NoteSelectionRunOut,
  NoteSpanOut,
}
export interface NoteCellOut {
  sourcePartIndex: number
  noteId: number
}
export interface LyricCellOut {
  sourcePartIndex: number
  noteId: number
  verse: number
}
export type { PartDeclarationOut, TextEditOut }
export type PartDeclarationModeOut = PartDeclarationOut['mode']
export type SymbolKindOut = 'abbreviation' | 'sectionLabel'
export type OccurrenceRoleOut = 'declaration' | 'reference'
export type DiagnosticSeverity = 'error' | 'warning'
export type TextAnchorOut = 'start' | 'middle' | 'end'
export type DominantBaselineOut = 'middle' | 'hanging' | 'ideographic'
export type FontWeightOut = 'normal' | 'bold'
export type FontFamilyOut = 'monospace' | 'sansSerif' | 'serif'
export type TransparentRectRoleOut =
  | 'measureClickTarget'
  | 'barNumberClickTarget'
  | 'sectionLabelBackground'
  | 'sectionLabelClickTarget'
  | 'noteClickTarget'
  | 'partLabelClickTarget'
  | 'lyricClickTarget'
  | 'lyricLabelClickTarget'
  | 'barLineClickTarget'
export type FontFamilyDefaultOut = 'serif' | 'sans_serif' | 'monospace'

// ---- shapes that need real field-name/value conversion ----

export interface DiagnosticViewZoneOut {
  severity: DiagnosticSeverity
  after_line_number: number
  messages: DiagnosticMessageOut[]
}

export interface NoteTimingOut {
  source_part_index: number
  note_id: number
  start_s: number
  end_s: number
}

export interface PartOut {
  abbreviation: string
  display_name: string
  has_lyrics: boolean
}

export interface MeasureSpanOut {
  start: number
  end: number
  view_zone_start: number
  section_label?: string
  start_line: number
  end_line: number
}

export interface SectionRangeOut {
  first_line: number
  last_line: number
  labels: string[]
}

export interface SequenceEntryOut {
  label: string
  start_measure_index: number
  end_measure_index: number
}

export interface SymbolOccurrenceOut {
  span: SpanOut
  hit_span: SpanOut
  role: OccurrenceRoleOut
}

export interface SymbolOut {
  name: string
  kind: SymbolKindOut
  occurrences: SymbolOccurrenceOut[]
}

export interface TextStyleDefaultsOut {
  font_size: number
  horizontal_padding_pt: number
  vertical_padding_pt: number
  bold: boolean
  italic: boolean
  underline: boolean
  font_family: FontFamilyDefaultOut
}

export interface MetadataDefaultsOut {
  row_height: number
  max_measures_per_system: number
  note_number_width: number
  parts_list_columns: number
  part_label_width_pt: number
  title: TextStyleDefaultsOut
  subtitle: TextStyleDefaultsOut
  author: TextStyleDefaultsOut
  sequence: TextStyleDefaultsOut
  part_legend: TextStyleDefaultsOut
  measure_number: TextStyleDefaultsOut
  section_label: TextStyleDefaultsOut
  part_label: TextStyleDefaultsOut
  page_number: TextStyleDefaultsOut
  lyrics: TextStyleDefaultsOut
  notes: TextStyleDefaultsOut
  chords: TextStyleDefaultsOut
  note_dash: TextStyleDefaultsOut
  merge_duplicate_measures_across_parts: boolean
  hide_resting_parts: boolean
  hide_system_dividers: boolean
  directive_row_offset_x: number
  directive_row_offset_y: number
}

export type TspanOut = {
  content: string
  bold: boolean
  italic: boolean
  underline: boolean
  font_size?: number
}

export type TagOut =
  | { type: 'measure'; index: number; end: number }
  | { type: 'barNumber'; index: number; end: number }
  | { type: 'sectionLabel'; label: string }
  | { type: 'note'; source_part_index: number; note_id: number }
  | {
      type: 'partLabel'
      source_part_index: number
      measure_index_start: number
      measure_index_end: number
    }
  | {
      type: 'lyric'
      source_part_index: number
      note_id: number
      verse: number
    }
  | {
      type: 'lyricLabel'
      source_part_index: number
      verse: number
      measure_index_start: number
      measure_index_end: number
    }
  | {
      type: 'barLine'
      measure_index_next: number | undefined
      measure_index_prev: number | undefined
    }

export type SvgKindOut =
  | {
      type: 'text'
      content: string
      font_size: number
      anchor: TextAnchorOut
      baseline: DominantBaselineOut
      font: FontFamilyOut
      weight: FontWeightOut
      italic: boolean
      underline: boolean
    }
  | { type: 'line'; x2: number; y2: number; stroke_width: number }
  | { type: 'circle'; r: number }
  | {
      type: 'path'
      control_x: number
      control_y: number
      end_x: number
      end_y: number
      stroke_width: number
    }
  | { type: 'rect'; width: number; height: number }
  | { type: 'errorRect'; width: number; height: number }
  | { type: 'playbackCursorRect'; width: number; height: number }
  | {
      type: 'transparentRect'
      width: number
      height: number
      role: TransparentRectRoleOut
    }
  | {
      type: 'textWithTspans'
      font_size: number
      anchor: TextAnchorOut
      baseline: DominantBaselineOut
      font: FontFamilyOut
      spans: TspanOut[]
    }
  | { type: 'group'; children: SvgElementOut[]; tag: TagOut | undefined }

export interface SvgElementOut {
  x: number
  y: number
  variant?: string
  kind: SvgKindOut
}

export interface SvgDocumentOut {
  width_pt: number
  height_pt: number
  elements: SvgElementOut[]
}

// ---- response envelope types (old `{ status, ... }` flat shape) ----

export type RenderResponse =
  | {
      status: 'ok'
      documents: SvgDocumentOut[]
      diagnostics: DiagnosticOut[]
      diagnostic_view_zones: DiagnosticViewZoneOut[]
    }
  | {
      status: 'err'
      diagnostics: DiagnosticOut[]
      diagnostic_view_zones: DiagnosticViewZoneOut[]
    }

export type ListPartsResponse =
  | { status: 'ok'; parts: PartOut[]; declarations: PartDeclarationOut[] }
  | { status: 'err'; diagnostics: DiagnosticOut[] }

export type ListPartDeclarationsResponse =
  | { status: 'ok'; declarations: PartDeclarationOut[] }
  | { status: 'err'; diagnostics: DiagnosticOut[] }

export type ListSymbolsResponse =
  | { status: 'ok'; symbols: SymbolOut[] }
  | { status: 'err'; diagnostics: DiagnosticOut[] }

export type RenameSymbolResponse =
  | { status: 'ok'; edits: TextEditOut[] }
  | { status: 'err'; diagnostics: DiagnosticOut[] }

export type MeasureAtOffsetResponse =
  | { status: 'ok'; measure_index: number }
  | { status: 'notInMeasure' }

export type ListNoteSpansResponse =
  | { status: 'ok'; spans: NoteSpanOut[] }
  | { status: 'err' }

export type ListLyricSpansResponse =
  | { status: 'ok'; spans: LyricSpanOut[] }
  | { status: 'err' }

export type ListMeasureSpansResponse =
  | {
      status: 'ok'
      spans: MeasureSpanOut[]
      section_ranges: SectionRangeOut[]
      sequence_entries: SequenceEntryOut[]
    }
  | { status: 'err' }

export type GroupNoteSelectionResponse =
  | { status: 'ok'; runs: NoteSelectionRunOut[] }
  | { status: 'err' }

export type GroupLyricSelectionResponse =
  | { status: 'ok'; runs: LyricSelectionRunOut[] }
  | { status: 'err' }

export type GenerateWavResponse =
  | { status: 'ok'; wav: Uint8Array }
  | { status: 'err'; diagnostics: DiagnosticOut[] }
export type GenerateMidiResponse =
  | { status: 'ok'; midi: Uint8Array }
  | { status: 'err'; diagnostics: DiagnosticOut[] }
export type GeneratePdfResponse =
  | { status: 'ok'; pdf: Uint8Array }
  | { status: 'err'; diagnostics: DiagnosticOut[] }
export type GenerateMp3Response =
  | { status: 'ok'; mp3: Uint8Array }
  | { status: 'err'; diagnostics: DiagnosticOut[] }
export type GenerateSplitPdfsResponse =
  | { status: 'ok'; zip: Uint8Array }
  | { status: 'err'; diagnostics: DiagnosticOut[] }
export type GenerateSplitMidisResponse =
  | { status: 'ok'; zip: Uint8Array }
  | { status: 'err'; diagnostics: DiagnosticOut[] }
export type GenerateSplitWavsResponse =
  | { status: 'ok'; zip: Uint8Array }
  | { status: 'err'; diagnostics: DiagnosticOut[] }
export type GenerateSplitMp3sResponse =
  | { status: 'ok'; zip: Uint8Array }
  | { status: 'err'; diagnostics: DiagnosticOut[] }

export type NoteTimingsResponse =
  | { status: 'ok'; timings: NoteTimingOut[] }
  | { status: 'err'; diagnostics: DiagnosticOut[] }
export type ResolveSelectionRangeResponse =
  | { status: 'ok'; note_cells: NoteCellOut[]; lyric_cells: LyricCellOut[] }
  | { status: 'err' }
