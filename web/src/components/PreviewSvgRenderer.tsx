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
