// Conversion helpers between the wit-bindgen/jco generated shapes and this
// package's old (`tsify`-generated) output shapes — split out of
// `jianpuWasm.ts` purely to stay under the 400-line-per-file cap. SVG-
// document conversion lives in the sibling `jianpuWasmConvertSvg.ts`
// (re-exported below) for the same reason.
import type {
  ClickableElementId as WitClickableElementId,
  Diagnostic as WitDiagnostic,
  DiagnosticViewZone as WitDiagnosticViewZone,
  FontFamilyDefault as WitFontFamilyDefault,
  MetadataDefaults as WitMetadataDefaults,
  NoteTiming as WitNoteTiming,
  Part as WitPart,
  Symbol as WitSymbol,
  SymbolKind as WitSymbolKind,
} from '../../crates/jianpu-wasm/pkg-component/jianpu_wasm.js'
import type { ClickableElementId } from './components/clickableElementId'
import type {
  DiagnosticOut,
  DiagnosticViewZoneOut,
  FontFamilyDefaultOut,
  MeasureSpanOut,
  MetadataDefaultsOut,
  NoteTimingOut,
  PartOut,
  SectionRangeOut,
  SequenceEntryOut,
  SymbolKindOut,
  SymbolOut,
  TextStyleDefaultsOut,
} from './jianpuWasmTypes'

export { convertSvgDocument } from './jianpuWasmConvertSvg'

function opt<T>(v: T | null | undefined): T | undefined {
  return v ?? undefined
}

function convertViewZone(z: WitDiagnosticViewZone): DiagnosticViewZoneOut {
  return {
    severity: z.severity,
    after_line_number: z.afterLineNumber,
    messages: z.messages,
  }
}

function convertPart(p: WitPart): PartOut {
  return {
    abbreviation: p.abbreviation,
    display_name: p.displayName,
    has_lyrics: p.hasLyrics,
  }
}

function convertMeasureSpan(s: {
  start: number
  end: number
  viewZoneStart: number
  sectionLabel?: string
  startLine: number
  endLine: number
}): MeasureSpanOut {
  return {
    start: s.start,
    end: s.end,
    view_zone_start: s.viewZoneStart,
    section_label: s.sectionLabel,
    start_line: s.startLine,
    end_line: s.endLine,
  }
}

function convertSectionRange(s: {
  firstLine: number
  lastLine: number
  labels: string[]
}): SectionRangeOut {
  return { first_line: s.firstLine, last_line: s.lastLine, labels: s.labels }
}

function convertSequenceEntry(s: {
  label: string
  startMeasureIndex: number
  endMeasureIndex: number
}): SequenceEntryOut {
  return {
    label: s.label,
    start_measure_index: s.startMeasureIndex,
    end_measure_index: s.endMeasureIndex,
  }
}

function convertNoteTiming(t: WitNoteTiming): NoteTimingOut {
  return {
    source_part_index: t.sourcePartIndex,
    note_id: t.noteId,
    start_s: t.startS,
    end_s: t.endS,
  }
}

const SYMBOL_KIND_FROM_WIT: Record<WitSymbolKind, SymbolKindOut> = {
  abbreviation: 'abbreviation',
  'section-label': 'sectionLabel',
}
const SYMBOL_KIND_TO_WIT: Record<SymbolKindOut, WitSymbolKind> = {
  abbreviation: 'abbreviation',
  sectionLabel: 'section-label',
}

function convertSymbol(s: WitSymbol): SymbolOut {
  return {
    name: s.name,
    kind: SYMBOL_KIND_FROM_WIT[s.kind],
    occurrences: s.occurrences.map((o) => ({
      span: o.span,
      hit_span: o.hitSpan,
      role: o.role,
    })),
  }
}

const FONT_FAMILY_DEFAULT_FROM_WIT: Record<
  WitFontFamilyDefault,
  FontFamilyDefaultOut
> = {
  serif: 'serif',
  'sans-serif': 'sans_serif',
  monospace: 'monospace',
}

function convertTextStyleDefaults(t: {
  fontSize: number
  horizontalPaddingPt: number
  verticalPaddingPt: number
  bold: boolean
  italic: boolean
  underline: boolean
  fontFamily: WitFontFamilyDefault
}): TextStyleDefaultsOut {
  return {
    font_size: t.fontSize,
    horizontal_padding_pt: t.horizontalPaddingPt,
    vertical_padding_pt: t.verticalPaddingPt,
    bold: t.bold,
    italic: t.italic,
    underline: t.underline,
    font_family: FONT_FAMILY_DEFAULT_FROM_WIT[t.fontFamily],
  }
}

function convertMetadataDefaults(m: WitMetadataDefaults): MetadataDefaultsOut {
  return {
    row_height: m.rowHeight,
    max_measures_per_system: m.maxMeasuresPerSystem,
    note_number_width: m.noteNumberWidth,
    parts_list_columns: m.partsListColumns,
    part_label_width_pt: m.partLabelWidthPt,
    title: convertTextStyleDefaults(m.title),
    subtitle: convertTextStyleDefaults(m.subtitle),
    author: convertTextStyleDefaults(m.author),
    sequence: convertTextStyleDefaults(m.sequence),
    part_legend: convertTextStyleDefaults(m.partLegend),
    measure_number: convertTextStyleDefaults(m.measureNumber),
    section_label: convertTextStyleDefaults(m.sectionLabel),
    part_label: convertTextStyleDefaults(m.partLabel),
    page_number: convertTextStyleDefaults(m.pageNumber),
    lyrics: convertTextStyleDefaults(m.lyrics),
    notes: convertTextStyleDefaults(m.notes),
    chords: convertTextStyleDefaults(m.chords),
    note_dash: convertTextStyleDefaults(m.noteDash),
    merge_duplicate_measures_across_parts: m.mergeDuplicateMeasuresAcrossParts,
    hide_resting_parts: m.hideRestingParts,
    hide_system_dividers: m.hideSystemDividers,
    directive_row_offset_x: m.directiveRowOffsetX,
    directive_row_offset_y: m.directiveRowOffsetY,
  }
}

// jco's generated `.d.ts` declares each variant-case's `val` payload
// interface twice under the same name — once as the flat record, once
// merged with `{ tag, val }` for the discriminated-union member — which
// TypeScript's declaration merging combines into one (self-referential)
// type wider than the plain payload record actually needs. A plain
// `{ tag, val }` object literal (the only shape jco's runtime actually
// reads) doesn't structurally satisfy that merged type, so this builds the
// value as `unknown` and asserts it into `WitClickableElementId` once at
// the end rather than fighting the merge on every case.
function convertClickableElementIdToWit(
  id: ClickableElementId,
): WitClickableElementId {
  const value = ((): unknown => {
    switch (id.kind) {
      case 'note':
        return {
          tag: 'note',
          val: { sourcePartIndex: id.sourcePartIndex, noteId: id.noteId },
        }
      case 'lyric':
        return {
          tag: 'lyric',
          val: {
            sourcePartIndex: id.sourcePartIndex,
            noteId: id.noteId,
            verse: id.verse,
          },
        }
      case 'measure':
        return {
          tag: 'measure',
          val: {
            measureIndexStart: id.measureIndexStart,
            measureIndexEnd: id.measureIndexEnd,
          },
        }
      case 'partLabel':
        return {
          tag: 'part-label',
          val: {
            sourcePartIndex: id.sourcePartIndex,
            measureIndexStart: id.measureIndexStart,
            measureIndexEnd: id.measureIndexEnd,
          },
        }
      case 'lyricLabel':
        return {
          tag: 'lyric-label',
          val: {
            sourcePartIndex: id.sourcePartIndex,
            verse: id.verse,
            measureIndexStart: id.measureIndexStart,
            measureIndexEnd: id.measureIndexEnd,
          },
        }
    }
  })()
  return value as WitClickableElementId
}

function diagnosticsErrOk<TWitOk, TOk extends object>(
  resp:
    | { tag: 'ok'; val: TWitOk }
    | { tag: 'err'; val: { diagnostics: WitDiagnostic[] } },
  convertOk: (v: TWitOk) => TOk,
): ({ status: 'ok' } & TOk) | { status: 'err'; diagnostics: DiagnosticOut[] } {
  if (resp.tag === 'ok') {
    return { status: 'ok', ...convertOk(resp.val) }
  }
  return { status: 'err', diagnostics: resp.val.diagnostics }
}

export {
  convertClickableElementIdToWit,
  convertMeasureSpan,
  convertMetadataDefaults,
  convertNoteTiming,
  convertPart,
  convertSectionRange,
  convertSequenceEntry,
  convertSymbol,
  convertViewZone,
  diagnosticsErrOk,
  opt,
  SYMBOL_KIND_TO_WIT,
}
