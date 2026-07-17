import type { SvgDocumentOut, SvgElementOut } from 'jianpu-wasm'
import type { ReactNode } from 'react'

function transparentRectRoleToDataVariant(
  role: 'measureClickTarget' | 'sectionLabelBackground',
): string {
  switch (role) {
    case 'measureClickTarget':
      return 'measure-click-target-rect'
    case 'sectionLabelBackground':
      return 'section-label-bg'
  }
}

// Vector Segno glyph, traced from a 190x190 viewBox. Adapted from
// "Music symbol Segno.svg" by Xavier enc (Wikimedia Commons), licensed
// CC BY-SA 3.0 / GFDL: https://commons.wikimedia.org/wiki/File:Music_symbol_Segno.svg
const SEGNO_GLYPH_PATH =
  'M162.542,147.629c0,24.913-15.023,37.37-45.072,37.37c-19.359,0-29.039-6.555-29.039-19.662' +
  'c0-5.346,2.094-9.933,6.276-13.764c4.185-3.833,9-5.747,14.444-5.747c5.85,0,10.765,1.989,14.746,5.974' +
  'c3.985,3.982,5.975,8.898,5.975,14.746c0,7.159-3.063,11.678-9.518,15.208c15.323-3.063,20.373-10.965,20.373-18.028' +
  'c0-9.894-6.429-17.883-20.867-27.772c-7.15-4.603-17.867-11.437-32.146-20.499l-42.125,56.929H29.89l47.295-63.913' +
  'c-12.863-7.794-23.734-16.813-32.621-27.066C33.157,68.19,27.458,55.179,27.458,42.371c0-24.913,15.023-37.37,45.07-37.37' +
  'c19.361,0,29.041,6.555,29.041,19.662c0,5.345-2.094,9.934-6.276,13.764c-4.187,3.834-9,5.747-14.444,5.747' +
  'c-5.85,0-10.765-1.989-14.746-5.974c-3.984-3.982-5.975-8.898-5.975-14.748c0-7.158,3.064-11.676,9.518-15.207' +
  'C54.321,11.31,49.272,19.21,49.272,26.274c0,9.893,6.43,17.882,20.869,27.771c7.149,4.604,17.865,11.438,32.144,20.5l42.125-56.928' +
  'h15.7l-47.295,63.914c12.859,7.792,23.734,16.813,32.619,27.066C156.843,121.81,162.542,134.821,162.542,147.629z M55.44,120.976' +
  'c0-6.969-5.65-12.619-12.621-12.619c-6.969,0-12.619,5.65-12.619,12.619c0,6.972,5.65,12.621,12.619,12.621' +
  'C49.79,133.597,55.44,127.946,55.44,120.976z M134.562,69.022c0,6.97,5.649,12.621,12.619,12.621c6.971,0,12.62-5.651,12.62-12.621' +
  'c0-6.971-5.649-12.619-12.62-12.619C140.211,56.403,134.562,62.052,134.562,69.022z'

function renderSvgElement(el: SvgElementOut, key: number): ReactNode {
  const { kind } = el
  switch (kind.type) {
    case 'text':
      return (
        <text
          key={key}
          x={el.x}
          y={el.y}
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
          fontFamily="sans-serif"
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
          rx={2}
          style={{ cursor: 'pointer' }}
        />
      )
    case 'group': {
      const measureIndex =
        kind.tag?.type === 'measure' ? kind.tag.index : undefined
      const measureIndexEnd =
        kind.tag?.type === 'measure' ? kind.tag.end : undefined
      const sectionLabel =
        kind.tag?.type === 'sectionLabel' ? kind.tag.label : undefined
      return (
        <g
          key={key}
          data-tag={
            measureIndex !== undefined
              ? 'measure'
              : sectionLabel !== undefined
                ? 'section-label'
                : undefined
          }
          data-measure-index={measureIndex}
          data-measure-index-end={measureIndexEnd}
          data-section-label={sectionLabel}
          style={
            measureIndex !== undefined || sectionLabel !== undefined
              ? { cursor: 'pointer' }
              : undefined
          }
        >
          {kind.children.map((child, i) => renderSvgElement(child, i))}
        </g>
      )
    }
    case 'segnoGlyph':
      return (
        <g
          key={key}
          transform={`translate(${el.x},${el.y}) scale(${kind.size / 190})`}
          data-variant="segno"
        >
          <path d={SEGNO_GLYPH_PATH} />
        </g>
      )
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
