import type { ReactNode } from 'react'
import fontsManifest from '../../../fonts/fonts.json'
import { DATA_VARIANT } from '../dataVariant'
import type {
  FontFamilyOut,
  SvgDocumentOut,
  SvgElementOut,
  TagOut,
  TransparentRectRoleOut,
} from '../jianpuWasm'

// `FontFamily::SansSerif`'s backing font — the default role for the
// directive line (bar number, section label, key/bpm/time signature,
// navigation markers), part legend, and footer, but overridable per-kind via
// `Metadata::*.font_family` (see `textFontFamily` below), mirroring
// `DIRECTIVE_LINE_FONT_FAMILY` in `src/serializer/mod.rs` (the Rust-side
// serializer backing exported .svg files and PDF export). Loaded via the
// `@font-face` rules injected by `injectFontFaces` (see src/injectFontFaces.ts),
// which point at the same font file bundled for PDF export (see
// `set_sans_serif_family` in src/pdf.rs) — instead of the generic
// `sans-serif` alias, so glyph widths stay consistent across viewers that
// have the font available. See `fonts/fonts.json` (this constant's source)
// and Task 1 of PLAN-section-label-engraving-quality.md.
const DIRECTIVE_LINE_FONT_FAMILY = fontsManifest.sansSerif.familyCss

// `FontFamily::Serif`'s backing font — the default role for the song title,
// subtitle, author, and lyric syllables/lines, but likewise overridable
// per-kind — is pinned to a separate, typically more calligraphic font
// instead. Mirrors `SERIF_FONT_FAMILY` in `src/serializer/mod.rs`.
const SERIF_FONT_FAMILY = fontsManifest.serif.familyCss

/** Resolves an element's `FontFamilyOut` (`text`'s `font`, or
 * `textWithTspans`'s own `font` — see `Metadata::measure_number_style`/
 * `section_label_style`/`sequence`'s `font_family`) to the CSS stack it
 * should render with. */
function textFontFamily(font: FontFamilyOut): string {
  switch (font) {
    case 'monospace':
      return 'monospace'
    case 'sansSerif':
      return DIRECTIVE_LINE_FONT_FAMILY
    case 'serif':
      return SERIF_FONT_FAMILY
  }
}

function transparentRectRoleToDataVariant(
  role: TransparentRectRoleOut,
): string {
  return DATA_VARIANT[role]
}

interface GroupTagAttrs {
  dataTag?: string
  dataMeasureIndex?: number
  dataMeasureIndexEnd?: number
  dataSectionLabel?: string
  dataPartIndex?: number
  dataNoteId?: number
  dataVerse?: number
  dataMeasureIndexStart?: number
  dataMeasureIndexNext?: number
  dataMeasureIndexPrev?: number
  cursor: boolean
}

function groupAttrsForTag(tag: TagOut | undefined): GroupTagAttrs {
  if (!tag) return { cursor: false }
  switch (tag.type) {
    case 'measure':
      return {
        dataTag: 'measure',
        dataMeasureIndex: tag.index,
        dataMeasureIndexEnd: tag.end,
        cursor: true,
      }
    case 'barNumber':
      return {
        dataTag: 'bar-number',
        dataMeasureIndex: tag.index,
        dataMeasureIndexEnd: tag.end,
        cursor: true,
      }
    case 'sectionLabel':
      return {
        dataTag: 'section-label',
        dataSectionLabel: tag.label,
        cursor: true,
      }
    case 'note':
      return {
        dataTag: 'note',
        dataPartIndex: tag.source_part_index,
        dataNoteId: tag.note_id,
        cursor: false,
      }
    case 'partLabel':
      return {
        dataTag: 'part-label',
        dataPartIndex: tag.source_part_index,
        dataMeasureIndexStart: tag.measure_index_start,
        dataMeasureIndexEnd: tag.measure_index_end,
        cursor: true,
      }
    case 'lyric':
      return {
        dataTag: 'lyric',
        dataPartIndex: tag.source_part_index,
        dataNoteId: tag.note_id,
        dataVerse: tag.verse,
        cursor: false,
      }
    case 'lyricLabel':
      return {
        dataTag: 'lyric-label',
        dataPartIndex: tag.source_part_index,
        dataVerse: tag.verse,
        dataMeasureIndexStart: tag.measure_index_start,
        dataMeasureIndexEnd: tag.measure_index_end,
        cursor: true,
      }
    case 'barLine':
      return {
        dataTag: 'bar-line',
        dataMeasureIndexNext: tag.measure_index_next,
        dataMeasureIndexPrev: tag.measure_index_prev,
        cursor: true,
      }
    default: {
      const exhaustiveCheck: never = tag
      throw new Error(
        `Unhandled Tag variant: ${JSON.stringify(exhaustiveCheck)}`,
      )
    }
  }
}

function renderSvgElement(el: SvgElementOut, key: number): ReactNode {
  const { kind } = el
  switch (kind.type) {
    case 'text':
      return (
        <text
          key={key}
          x={el.x}
          y={el.y}
          data-variant={el.variant}
          fontSize={kind.font_size}
          textAnchor={
            kind.anchor === 'start'
              ? 'start'
              : kind.anchor === 'middle'
                ? 'middle'
                : 'end'
          }
          dominantBaseline={
            kind.baseline === 'middle'
              ? 'middle'
              : kind.baseline === 'hanging'
                ? 'hanging'
                : 'ideographic'
          }
          fontFamily={textFontFamily(kind.font)}
          fontWeight={kind.weight === 'normal' ? 'normal' : 'bold'}
          fontStyle={kind.italic ? 'italic' : undefined}
          textDecoration={kind.underline ? 'underline' : undefined}
        >
          {kind.content}
        </text>
      )
    case 'textWithTspans':
      return (
        <text
          key={key}
          x={el.x}
          y={el.y}
          data-variant={el.variant}
          fontSize={kind.font_size}
          textAnchor={
            kind.anchor === 'start'
              ? 'start'
              : kind.anchor === 'middle'
                ? 'middle'
                : 'end'
          }
          dominantBaseline={
            kind.baseline === 'middle'
              ? 'middle'
              : kind.baseline === 'hanging'
                ? 'hanging'
                : 'ideographic'
          }
          fontFamily={textFontFamily(kind.font)}
        >
          {kind.spans.map((span, spanIndex) => (
            <tspan
              // biome-ignore lint/suspicious/noArrayIndexKey: tspans have no stable identifier
              key={spanIndex}
              fontWeight={span.bold ? 'bold' : undefined}
              fontStyle={span.italic ? 'italic' : undefined}
              textDecoration={span.underline ? 'underline' : undefined}
              fontSize={span.font_size ?? undefined}
            >
              {span.content}
            </tspan>
          ))}
        </text>
      )
    case 'line':
      return (
        <line
          key={key}
          x1={el.x}
          y1={el.y}
          x2={kind.x2}
          y2={kind.y2}
          stroke="black"
          strokeWidth={kind.stroke_width}
        />
      )
    case 'circle':
      return <circle key={key} cx={el.x} cy={el.y} r={kind.r} fill="black" />
    case 'path':
      return (
        <path
          key={key}
          d={`M ${el.x} ${el.y} Q ${kind.control_x} ${kind.control_y} ${kind.end_x} ${kind.end_y}`}
          fill="none"
          stroke="black"
          strokeWidth={kind.stroke_width}
        />
      )
    case 'rect':
      return (
        <rect
          key={key}
          data-testid="measure-highlight"
          x={el.x}
          y={el.y}
          width={kind.width}
          height={kind.height}
          fill="rgba(255,200,0,0.25)"
          rx={2}
        />
      )
    case 'errorRect':
      return (
        <rect
          key={key}
          data-testid="error-highlight"
          x={el.x}
          y={el.y}
          width={kind.width}
          height={kind.height}
          fill="rgba(255,0,0,0.15)"
          rx={2}
        />
      )
    case 'transparentRect':
      return (
        <rect
          key={key}
          x={el.x}
          y={el.y}
          width={kind.width}
          height={kind.height}
          data-variant={transparentRectRoleToDataVariant(kind.role)}
          fill="transparent"
          stroke={kind.role === 'sectionLabelBackground' ? 'black' : undefined}
          strokeWidth={kind.role === 'sectionLabelBackground' ? 1 : undefined}
          rx={2}
          style={{
            cursor:
              kind.role === 'barLineClickTarget' ? 'col-resize' : 'pointer',
          }}
        />
      )
    case 'playbackCursorRect':
      return (
        <rect
          key={key}
          data-variant={DATA_VARIANT.playbackCursorRect}
          x={el.x}
          y={el.y}
          width={kind.width}
          height={kind.height}
          fill="transparent"
          rx={2}
          style={{ pointerEvents: 'none' }}
        />
      )
    case 'group': {
      const attrs = groupAttrsForTag(kind.tag)
      return (
        <g
          key={key}
          data-tag={attrs.dataTag}
          data-measure-index={attrs.dataMeasureIndex}
          data-measure-index-end={attrs.dataMeasureIndexEnd}
          data-section-label={attrs.dataSectionLabel}
          data-part-index={attrs.dataPartIndex}
          data-note-id={attrs.dataNoteId}
          data-verse={attrs.dataVerse}
          data-measure-index-start={attrs.dataMeasureIndexStart}
          data-measure-index-next={attrs.dataMeasureIndexNext}
          data-measure-index-prev={attrs.dataMeasureIndexPrev}
          style={attrs.cursor ? { cursor: 'pointer' } : undefined}
        >
          {kind.children.map((child, i) => renderSvgElement(child, i))}
        </g>
      )
    }
  }
}

export function renderSvgDocument(doc: SvgDocumentOut, key: number): ReactNode {
  return (
    // biome-ignore lint/a11y/noSvgWithoutTitle: synthesized score SVG; title would be redundant with surrounding page context
    <svg
      key={key}
      xmlns="http://www.w3.org/2000/svg"
      width="210mm"
      height="297mm"
      viewBox={`0 0 ${Math.round(doc.width_pt)} ${Math.round(doc.height_pt)}`}
    >
      {doc.elements.map((el, i) => renderSvgElement(el, i))}
    </svg>
  )
}
