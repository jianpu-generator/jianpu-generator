import type { ReactNode } from 'react'
import fontsManifest from '../../../fonts/fonts.json'
import { DATA_VARIANT, groupAttrsForTag } from '../dataAttributes'
import type {
  FontFamily,
  SvgDocument,
  TransparentRectRole,
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

/** Resolves an element's `FontFamily` (`text`'s `font`, or
 * `text-with-tspans`'s own `font` — see `Metadata::measure_number_style`/
 * `section_label_style`/`sequence`'s `font_family`) to the CSS stack it
 * should render with. */
function textFontFamily(font: FontFamily): string {
  switch (font) {
    case 'monospace':
      return 'monospace'
    case 'sans-serif':
      return DIRECTIVE_LINE_FONT_FAMILY
    case 'serif':
      return SERIF_FONT_FAMILY
  }
}

function transparentRectRoleToDataVariant(role: TransparentRectRole): string {
  return DATA_VARIANT[role]
}

/** Renders `doc.elements[index]` — the Rust side flattens the element tree
 * into this arena in pre-order, with each group naming its children by
 * index (`SvgGroupKind.childIndices`). */
function renderSvgElement(doc: SvgDocument, index: number): ReactNode {
  // biome-ignore lint/style/noNonNullAssertion: index always comes from the arena's own childIndices/rootElementIndices
  const el = doc.elements[index]!
  const key = index
  const { tag, val: kind } = el.kind
  switch (tag) {
    case 'text':
      return (
        <text
          key={key}
          x={el.x}
          y={el.y}
          data-variant={el.variantTag}
          fontSize={kind.fontSize}
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
    case 'text-with-tspans':
      return (
        <text
          key={key}
          x={el.x}
          y={el.y}
          data-variant={el.variantTag}
          fontSize={kind.fontSize}
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
              fontSize={span.fontSize}
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
          strokeWidth={kind.strokeWidth}
        />
      )
    case 'circle':
      return <circle key={key} cx={el.x} cy={el.y} r={kind.r} fill="black" />
    case 'path':
      return (
        <path
          key={key}
          d={`M ${el.x} ${el.y} Q ${kind.controlX} ${kind.controlY} ${kind.endX} ${kind.endY}`}
          fill="none"
          stroke="black"
          strokeWidth={kind.strokeWidth}
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
    case 'error-rect':
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
    case 'transparent-rect':
      return (
        <rect
          key={key}
          x={el.x}
          y={el.y}
          width={kind.width}
          height={kind.height}
          data-variant={transparentRectRoleToDataVariant(kind.role)}
          fill="transparent"
          stroke={
            kind.role === 'section-label-background' ? 'black' : undefined
          }
          strokeWidth={kind.role === 'section-label-background' ? 1 : undefined}
          rx={2}
          style={{
            cursor:
              kind.role === 'bar-line-click-target' ? 'col-resize' : 'pointer',
          }}
        />
      )
    case 'playback-cursor-rect':
      return (
        <rect
          key={key}
          data-variant={DATA_VARIANT['playback-cursor-rect']}
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
          {Array.from(kind.childIndices, (child) =>
            renderSvgElement(doc, child),
          )}
        </g>
      )
    }
  }
}

export function renderSvgDocument(doc: SvgDocument, key: number): ReactNode {
  return (
    // biome-ignore lint/a11y/noSvgWithoutTitle: synthesized score SVG; title would be redundant with surrounding page context
    <svg
      key={key}
      xmlns="http://www.w3.org/2000/svg"
      width="210mm"
      height="297mm"
      viewBox={`0 0 ${Math.round(doc.widthPt)} ${Math.round(doc.heightPt)}`}
    >
      {Array.from(doc.rootElementIndices, (index) =>
        renderSvgElement(doc, index),
      )}
    </svg>
  )
}
