// SVG-document conversion helpers between the wit-bindgen/jco generated
// shapes and this package's old (`tsify`-generated) output shapes — split
// out of `jianpuWasmConvert.ts` (which re-exports `convertSvgDocument`)
// purely to stay under the 400-line-per-file cap.
import type {
  FontFamily as WitFontFamily,
  SvgDocument as WitSvgDocument,
  SvgElement as WitSvgElement,
  SvgKind as WitSvgKind,
  Tag as WitTag,
  TransparentRectRole as WitTransparentRectRole,
  Tspan as WitTspan,
} from '../../crates/jianpu-wasm/pkg-component/jianpu_wasm.js'
import type {
  FontFamilyOut,
  SvgDocumentOut,
  SvgElementOut,
  SvgKindOut,
  TagOut,
  TransparentRectRoleOut,
  TspanOut,
} from './jianpuWasmTypes'

const FONT_FAMILY_FROM_WIT: Record<WitFontFamily, FontFamilyOut> = {
  monospace: 'monospace',
  'sans-serif': 'sansSerif',
  serif: 'serif',
}

const TRANSPARENT_RECT_ROLE_FROM_WIT: Record<
  WitTransparentRectRole,
  TransparentRectRoleOut
> = {
  'measure-click-target': 'measureClickTarget',
  'bar-number-click-target': 'barNumberClickTarget',
  'section-label-background': 'sectionLabelBackground',
  'section-label-click-target': 'sectionLabelClickTarget',
  'note-click-target': 'noteClickTarget',
  'part-label-click-target': 'partLabelClickTarget',
  'lyric-click-target': 'lyricClickTarget',
  'lyric-label-click-target': 'lyricLabelClickTarget',
  'bar-line-click-target': 'barLineClickTarget',
}

function convertTspan(t: WitTspan): TspanOut {
  return {
    content: t.content,
    bold: t.bold,
    italic: t.italic,
    underline: t.underline,
    font_size: t.fontSize,
  }
}

function convertTag(tag: WitTag): TagOut {
  switch (tag.tag) {
    case 'measure':
      return { type: 'measure', index: tag.val.index, end: tag.val.end }
    case 'bar-number':
      return { type: 'barNumber', index: tag.val.index, end: tag.val.end }
    case 'section-label':
      return { type: 'sectionLabel', label: tag.val.label }
    case 'note':
      return {
        type: 'note',
        source_part_index: tag.val.sourcePartIndex,
        note_id: tag.val.noteId,
      }
    case 'part-label':
      return {
        type: 'partLabel',
        source_part_index: tag.val.sourcePartIndex,
        measure_index_start: tag.val.measureIndexStart,
        measure_index_end: tag.val.measureIndexEnd,
      }
    case 'lyric':
      return {
        type: 'lyric',
        source_part_index: tag.val.sourcePartIndex,
        note_id: tag.val.noteId,
        verse: tag.val.verse,
      }
    case 'lyric-label':
      return {
        type: 'lyricLabel',
        source_part_index: tag.val.sourcePartIndex,
        verse: tag.val.verse,
        measure_index_start: tag.val.measureIndexStart,
        measure_index_end: tag.val.measureIndexEnd,
      }
    case 'bar-line':
      return {
        type: 'barLine',
        measure_index_next: tag.val.measureIndexNext,
        measure_index_prev: tag.val.measureIndexPrev,
      }
  }
}

function convertSvgKind(
  arena: readonly WitSvgElement[],
  kind: WitSvgKind,
): SvgKindOut {
  switch (kind.tag) {
    case 'text':
      return {
        type: 'text',
        content: kind.val.content,
        font_size: kind.val.fontSize,
        anchor: kind.val.anchor,
        baseline: kind.val.baseline,
        font: FONT_FAMILY_FROM_WIT[kind.val.font],
        weight: kind.val.weight,
        italic: kind.val.italic,
        underline: kind.val.underline,
      }
    case 'line':
      return {
        type: 'line',
        x2: kind.val.x2,
        y2: kind.val.y2,
        stroke_width: kind.val.strokeWidth,
      }
    case 'circle':
      return { type: 'circle', r: kind.val.r }
    case 'path':
      return {
        type: 'path',
        control_x: kind.val.controlX,
        control_y: kind.val.controlY,
        end_x: kind.val.endX,
        end_y: kind.val.endY,
        stroke_width: kind.val.strokeWidth,
      }
    case 'rect':
      return { type: 'rect', width: kind.val.width, height: kind.val.height }
    case 'error-rect':
      return {
        type: 'errorRect',
        width: kind.val.width,
        height: kind.val.height,
      }
    case 'playback-cursor-rect':
      return {
        type: 'playbackCursorRect',
        width: kind.val.width,
        height: kind.val.height,
      }
    case 'transparent-rect':
      return {
        type: 'transparentRect',
        width: kind.val.width,
        height: kind.val.height,
        role: TRANSPARENT_RECT_ROLE_FROM_WIT[kind.val.role],
      }
    case 'text-with-tspans':
      return {
        type: 'textWithTspans',
        font_size: kind.val.fontSize,
        anchor: kind.val.anchor,
        baseline: kind.val.baseline,
        font: FONT_FAMILY_FROM_WIT[kind.val.font],
        spans: kind.val.spans.map(convertTspan),
      }
    case 'group':
      return {
        type: 'group',
        children: Array.from(kind.val.childIndices).map((i) =>
          convertSvgElement(arena, i),
        ),
        tag: kind.val.tag ? convertTag(kind.val.tag) : undefined,
      }
  }
}

function convertSvgElement(
  arena: readonly WitSvgElement[],
  index: number,
): SvgElementOut {
  // biome-ignore lint/style/noNonNullAssertion: index always comes from the arena's own child-indices/root-indices, produced by the Rust side's own pre-order flattening
  const el = arena[index]!
  return {
    x: el.x,
    y: el.y,
    variant: el.variantTag,
    kind: convertSvgKind(arena, el.kind),
  }
}

export function convertSvgDocument(doc: WitSvgDocument): SvgDocumentOut {
  return {
    width_pt: doc.widthPt,
    height_pt: doc.heightPt,
    elements: Array.from(doc.rootElementIndices).map((i) =>
      convertSvgElement(doc.elements, i),
    ),
  }
}
