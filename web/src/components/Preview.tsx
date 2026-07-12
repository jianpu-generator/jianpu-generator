import type { SvgDocumentOut, SvgElementOut } from 'jianpu-wasm'
import { type ReactNode, useEffect, useRef, useState } from 'react'

interface PreviewProps {
  documents: SvgDocumentOut[]
  highlightedDocuments?: SvgDocumentOut[]
  rendering: boolean
  audioGenerating?: boolean
  wavUrl?: string | null
  wavFilename?: string
  /** Elapsed-seconds offset of each measure boundary for `wavUrl`'s audio, length = measure count + 1. */
  measureTimes?: number[]
  /** Elapsed-seconds offset of each measure boundary within the selected range's audio, relative to the range start. */
  measureAudioTimes?: number[]
  /** The `<audio>` element currently playing the selected measure range, if any. */
  measureAudioElement?: HTMLAudioElement | null
  selectedMeasureRange?: { start: number; end: number } | null
  emptyMessage?: string
  toolbar?: ReactNode
  onMeasureRangeSelect?: (startIndex: number, endIndex: number) => void
  onSectionLabelClick?: (label: string) => void
}

function findMeasureSegmentAtTime(times: number[], t: number): number {
  for (let i = times.length - 2; i >= 0; i--) {
    if (t >= times[i]) return i
  }
  return 0
}

/**
 * Imperatively drives an SVG playhead `<rect>` across measures in sync with
 * `audio`'s playback position, using `measureTimes` (seconds per measure
 * boundary, offset by `measureIndexOffset`) to locate each measure's x/width
 * via its existing click-target rect. Runs outside React state/rendering
 * (rAF, direct attribute writes) since it updates every animation frame.
 */
function usePlayhead(
  containerRef: React.RefObject<HTMLDivElement | null>,
  audio: HTMLAudioElement | null | undefined,
  measureTimes: number[] | undefined,
  measureIndexOffset: number,
) {
  useEffect(() => {
    const container = containerRef.current
    if (!audio || !container || !measureTimes || measureTimes.length < 2) {
      return
    }

    const playhead = document.createElementNS(
      'http://www.w3.org/2000/svg',
      'rect',
    )
    playhead.setAttribute('data-testid', 'measure-playhead')
    playhead.setAttribute('width', '2')
    playhead.setAttribute('fill', 'rgba(220,38,38,0.85)')
    playhead.style.pointerEvents = 'none'
    let currentSvg: SVGSVGElement | null = null
    let rafId: number | null = null

    const updatePosition = () => {
      const t = audio.currentTime
      const segment = findMeasureSegmentAtTime(measureTimes, t)
      const measureIndex = measureIndexOffset + segment
      const group = container.querySelector<SVGGElement>(
        `[data-tag="measure"][data-measure-index="${measureIndex}"]`,
      )
      const targetRect = group?.querySelector<SVGRectElement>(
        'rect[data-variant="measure-click-target-rect"]',
      )
      if (!group || !targetRect) return
      const svg = group.closest('svg')
      if (svg && svg !== currentSvg) {
        playhead.remove()
        svg.appendChild(playhead)
        currentSvg = svg
      }
      const x = Number.parseFloat(targetRect.getAttribute('x') ?? '0')
      const y = Number.parseFloat(targetRect.getAttribute('y') ?? '0')
      const width = Number.parseFloat(targetRect.getAttribute('width') ?? '0')
      const height = Number.parseFloat(targetRect.getAttribute('height') ?? '0')
      const segStart = measureTimes[segment]
      const segEnd = measureTimes[segment + 1] ?? segStart
      const fraction =
        segEnd > segStart
          ? Math.min(1, Math.max(0, (t - segStart) / (segEnd - segStart)))
          : 0
      playhead.setAttribute('x', String(x + fraction * width))
      playhead.setAttribute('y', String(y))
      playhead.setAttribute('height', String(height))
    }

    const tick = () => {
      updatePosition()
      rafId = requestAnimationFrame(tick)
    }
    const start = () => {
      if (rafId === null) rafId = requestAnimationFrame(tick)
    }
    const stop = () => {
      if (rafId !== null) {
        cancelAnimationFrame(rafId)
        rafId = null
      }
      playhead.remove()
      currentSvg = null
    }

    audio.addEventListener('play', start)
    audio.addEventListener('pause', stop)
    audio.addEventListener('ended', stop)
    if (!audio.paused) start()

    return () => {
      audio.removeEventListener('play', start)
      audio.removeEventListener('pause', stop)
      audio.removeEventListener('ended', stop)
      stop()
    }
  }, [containerRef, audio, measureTimes, measureIndexOffset])
}

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

function getSectionLabelAtPoint(x: number, y: number): string | undefined {
  const el = document.elementFromPoint(x, y)
  if (!el) return undefined
  const group = el.closest('[data-tag="section-label"]')
  if (!group) return undefined
  return (group as HTMLElement).dataset.sectionLabel
}

function getMeasureAtPoint(x: number, y: number): number | undefined {
  const el = document.elementFromPoint(x, y)
  if (!el) return undefined
  const group = el.closest('[data-tag="measure"]')
  if (!group) return undefined
  const index = (group as HTMLElement).dataset.measureIndex
  if (index === undefined) return undefined
  const parsed = Number.parseInt(index, 10)
  return Number.isNaN(parsed) ? undefined : parsed
}

function applyDragHighlights(
  container: HTMLElement,
  start: number,
  current: number,
): void {
  const min = Math.min(start, current)
  const max = Math.max(start, current)
  for (const group of Array.from(
    container.querySelectorAll<HTMLElement>('[data-tag="measure"]'),
  )) {
    const index = Number.parseInt(group.dataset.measureIndex ?? '', 10)
    if (index >= min && index <= max) {
      group.dataset.dragSelected = ''
    } else {
      delete group.dataset.dragSelected
    }
  }
}

function clearDragHighlights(container: HTMLElement): void {
  for (const group of Array.from(
    container.querySelectorAll<HTMLElement>('[data-tag="measure"]'),
  )) {
    delete group.dataset.dragSelected
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

function renderSvgDocument(doc: SvgDocumentOut, key: number): ReactNode {
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

export function Preview({
  documents,
  highlightedDocuments = [],
  rendering,
  audioGenerating = false,
  wavUrl = null,
  wavFilename = 'audio.wav',
  measureTimes,
  measureAudioTimes,
  measureAudioElement,
  selectedMeasureRange,
  emptyMessage = 'No preview yet.',
  toolbar,
  onMeasureRangeSelect,
  onSectionLabelClick,
}: PreviewProps) {
  const previewPagesRef = useRef<HTMLDivElement>(null)
  const [audioElement, setAudioElement] = useState<HTMLAudioElement | null>(
    null,
  )

  usePlayhead(previewPagesRef, audioElement, measureTimes, 0)
  usePlayhead(
    previewPagesRef,
    measureAudioElement,
    measureAudioTimes,
    selectedMeasureRange?.start ?? 0,
  )
  const dragStateRef = useRef<{
    startIndex: number
    currentIndex: number
  } | null>(null)
  const onMeasureRangeSelectRef = useRef(onMeasureRangeSelect)
  onMeasureRangeSelectRef.current = onMeasureRangeSelect
  const onSectionLabelClickRef = useRef(onSectionLabelClick)
  onSectionLabelClickRef.current = onSectionLabelClick

  useEffect(() => {
    if (!audioGenerating) return
    if (audioElement && !audioElement.paused) {
      audioElement.pause()
    }
  }, [audioGenerating, audioElement])

  useEffect(() => {
    if (highlightedDocuments.length === 0) return

    const frameId = requestAnimationFrame(() => {
      const container = previewPagesRef.current
      if (!container) return

      const highlight = container.querySelector(
        '[data-testid="measure-highlight"]',
      )
      highlight?.scrollIntoView({
        block: 'center',
        inline: 'nearest',
      })
    })

    return () => cancelAnimationFrame(frameId)
  }, [highlightedDocuments])

  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      const dragState = dragStateRef.current
      if (!dragState) return
      const container = previewPagesRef.current
      if (!container) return
      const index = getMeasureAtPoint(e.clientX, e.clientY)
      if (index !== undefined) {
        dragState.currentIndex = index
        applyDragHighlights(
          container,
          dragState.startIndex,
          dragState.currentIndex,
        )
      }
    }

    const handleMouseUp = (e: MouseEvent) => {
      const dragState = dragStateRef.current
      if (!dragState) return
      const container = previewPagesRef.current
      if (container) {
        clearDragHighlights(container)
      }
      const index = getMeasureAtPoint(e.clientX, e.clientY)
      const finalIndex = index ?? dragState.currentIndex
      const min = Math.min(dragState.startIndex, finalIndex)
      const max = Math.max(dragState.startIndex, finalIndex)
      onMeasureRangeSelectRef.current?.(min, max)
      dragStateRef.current = null
    }

    document.addEventListener('mousemove', handleMouseMove)
    document.addEventListener('mouseup', handleMouseUp)
    return () => {
      document.removeEventListener('mousemove', handleMouseMove)
      document.removeEventListener('mouseup', handleMouseUp)
    }
  }, [])

  const activeDocs =
    highlightedDocuments.length > 0 ? highlightedDocuments : documents

  return (
    <div className="preview">
      {toolbar ? <div className="preview-toolbar">{toolbar}</div> : null}
      {wavUrl ? (
        <div
          className={
            audioGenerating
              ? 'preview-audio preview-audio--generating'
              : 'preview-audio'
          }
          aria-busy={audioGenerating || undefined}
        >
          {/* biome-ignore lint/a11y/useMediaCaption: synthesized score preview has no captions track */}
          <audio
            ref={setAudioElement}
            className="preview-audio-player"
            controls
            src={wavUrl}
            tabIndex={audioGenerating ? -1 : undefined}
          />
          <a
            className="preview-audio-download"
            href={wavUrl}
            download={wavFilename}
            tabIndex={audioGenerating ? -1 : undefined}
          >
            Download
          </a>
        </div>
      ) : null}
      <div className="preview-pages-wrapper">
        {/* biome-ignore lint/a11y/noStaticElementInteractions: drag-to-select measures uses mousedown, mousemove, mouseup — not a standard interactive role */}
        <div
          className="preview-pages"
          ref={previewPagesRef}
          onMouseDown={(e) => {
            const sectionLabel = getSectionLabelAtPoint(e.clientX, e.clientY)
            if (sectionLabel !== undefined) {
              onSectionLabelClickRef.current?.(sectionLabel)
              e.preventDefault()
              return
            }
            const index = getMeasureAtPoint(e.clientX, e.clientY)
            if (index === undefined) return
            dragStateRef.current = { startIndex: index, currentIndex: index }
            const container = previewPagesRef.current
            if (container) {
              applyDragHighlights(container, index, index)
            }
            e.preventDefault()
          }}
        >
          {documents.length === 0 &&
          highlightedDocuments.length === 0 &&
          !rendering ? (
            <p className="preview-empty">{emptyMessage}</p>
          ) : null}
          {activeDocs.map((doc, i) => (
            // biome-ignore lint/suspicious/noArrayIndexKey: pages have no stable identifier
            <div key={i} className="preview-page">
              {renderSvgDocument(doc, i)}
            </div>
          ))}
        </div>
        {rendering ? (
          <div
            className="preview-render-spinner"
            role="status"
            aria-label="Rendering"
          />
        ) : null}
      </div>
    </div>
  )
}
