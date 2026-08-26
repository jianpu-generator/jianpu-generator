import type { SvgDocumentOut, SvgElementOut, TagOut } from 'jianpu-wasm'
import type { ReactNode } from 'react'

// The directive line (bar number, section label, key/bpm/time signature,
// navigation markers) is the sole user of the `textWithTspans` case below.
// It's pinned to a specific font — loaded via the `@font-face` rule in
// index.css, which points at the same font file bundled for PDF export
// (see `set_sans_serif_family` in src/pdf.rs) — instead of the generic
// `sans-serif` alias, so glyph widths stay consistent across viewers that
// have the font available. See Task 1 of
// PLAN-section-label-engraving-quality.md.
const DIRECTIVE_LINE_FONT_FAMILY = '"Source Han Sans SC", sans-serif'

/** Stroke width (SVG units) of the invisible hit-line drawn over each bar
 * line — wide enough to be reliably hoverable/draggable (the real bar line
 * is only 0.5pt), narrow enough to stay inside its own measure-column
 * share (`MIN_MEASURE_WIDTH_PT` floors every measure at 24pt) and not eat
 * into a neighboring note's or part label's own click target. */
const BAR_LINE_HIT_WIDTH = 6

function transparentRectRoleToDataVariant(
  role:
    | 'measureClickTarget'
    | 'sectionLabelBackground'
    | 'sectionLabelClickTarget'
    | 'noteClickTarget'
    | 'partLabelClickTarget'
    | 'lyricClickTarget'
    | 'lyricLabelClickTarget'
    | 'barNumberClickTarget',
): string {
  switch (role) {
    case 'measureClickTarget':
      return 'measure-click-target-rect'
    case 'sectionLabelBackground':
      return 'section-label-bg'
    case 'sectionLabelClickTarget':
      return 'section-label-click-target-rect'
    case 'noteClickTarget':
      return 'note-click-target-rect'
    case 'partLabelClickTarget':
      return 'part-label-click-target-rect'
    case 'lyricClickTarget':
      return 'lyric-click-target-rect'
    case 'lyricLabelClickTarget':
      return 'lyric-label-click-target-rect'
    case 'barNumberClickTarget':
      return 'bar-number-click-target-rect'
  }
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
          fontFamily={kind.font === 'monospace' ? 'monospace' : 'sans-serif'}
          fontWeight={kind.weight === 'normal' ? 'normal' : 'bold'}
          fontStyle={kind.italic ? 'italic' : undefined}
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
          fontFamily={DIRECTIVE_LINE_FONT_FAMILY}
        >
          {kind.spans.map((span, spanIndex) => (
            <tspan
              // biome-ignore lint/suspicious/noArrayIndexKey: tspans have no stable identifier
              key={spanIndex}
              fontWeight={span.bold ? 'bold' : undefined}
              fontStyle={span.italic ? 'italic' : undefined}
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
          style={{ cursor: 'pointer' }}
        />
      )
    case 'playbackCursorRect':
      return (
        <rect
          key={key}
          data-variant="playback-cursor-rect"
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
          style={attrs.cursor ? { cursor: 'pointer' } : undefined}
        >
          {kind.children.map((child, i) => renderSvgElement(child, i))}
        </g>
      )
    }
  }
}

/** Bar lines render inline with everything else in `doc.elements`' original
 * order, so a note's or measure's own click-target rect (added later in that
 * order, per `render_new_renderer`'s element sequencing) normally paints
 * over — and wins hit-testing against — the thin bar-line stroke beneath it.
 * `renderSvgDocument` renders a second, invisible, wider hit-line for each
 * one *after* every other element, so it's reliably topmost for hover/click
 * regardless of where the bar line falls in the original sequence. Bar
 * lines are always emitted flat into `page.elements` (see
 * `render_bar_line`), never nested inside a `Group`, so a shallow scan is
 * enough. */
function collectBarLines(elements: SvgElementOut[]): SvgElementOut[] {
  return elements.filter(
    (el) => el.kind.type === 'line' && el.variant === 'bar-line',
  )
}

/** The invisible, wider drag handle drawn over a bar line (see
 * `collectBarLines`) — gives the divider a real hover/cursor affordance and
 * lets a mousedown here fall through past section-label/part-label/note hit
 * detection (all `elementFromPoint`-based) straight to `Preview.tsx`'s
 * measure-range fallback, so grabbing a bar line always starts a clean
 * measure-range drag instead of racing whatever note or label happens to
 * share that pixel. */
function renderBarLineDragHandle(el: SvgElementOut, key: number): ReactNode {
  if (el.kind.type !== 'line') return null
  return (
    <line
      key={key}
      x1={el.x}
      y1={el.y}
      x2={el.kind.x2}
      y2={el.kind.y2}
      stroke="transparent"
      strokeWidth={BAR_LINE_HIT_WIDTH}
      className="bar-line-drag-handle"
    />
  )
}

export function renderSvgDocument(doc: SvgDocumentOut, key: number): ReactNode {
  const barLines = collectBarLines(doc.elements)
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
      {barLines.map((el, i) => renderBarLineDragHandle(el, i))}
    </svg>
  )
}
