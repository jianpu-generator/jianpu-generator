import type { SvgDocumentOut, SvgElementOut } from 'jianpu-wasm'
import { type ReactNode, useEffect, useRef } from 'react'

interface PreviewProps {
  documents: SvgDocumentOut[]
  highlightedDocuments?: SvgDocumentOut[]
  rendering: boolean
  audioGenerating?: boolean
  wavUrl?: string | null
  audioAvailable?: boolean
  soundfontReady?: boolean
  onGenerateAudio?: () => void
  pdfAvailable?: boolean
  pdfFontsReady?: boolean
  pdfExporting?: boolean
  onExportPdf?: () => void
  splitPdfExporting?: boolean
  onExportSplitPdf?: () => void
  partsCount?: number
  emptyMessage?: string
  toolbar?: ReactNode
  onMeasureClick?: (measureIndex: number) => void
}

function renderSvgElement(
  el: SvgElementOut,
  key: number,
  onMeasureClick?: (measureIndex: number) => void,
): ReactNode {
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
    case 'line':
      return (
        <line
          key={key}
          x1={el.x}
          y1={el.y}
          x2={kind.x2}
          y2={kind.y2}
          data-variant={el.variant}
          stroke="black"
          strokeWidth={kind.stroke_width}
        />
      )
    case 'circle':
      return (
        <circle
          key={key}
          cx={el.x}
          cy={el.y}
          data-variant={el.variant}
          r={kind.r}
          fill="black"
        />
      )
    case 'path':
      return (
        <path
          key={key}
          d={`M ${el.x} ${el.y} Q ${kind.control_x} ${kind.control_y} ${kind.end_x} ${kind.end_y}`}
          data-variant={el.variant}
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
          data-variant={el.variant}
          fill="transparent"
          rx={2}
          style={{ cursor: 'pointer' }}
        />
      )
    case 'group': {
      const measureIndex =
        kind.tag?.type === 'measure' ? kind.tag.index : undefined
      return (
        // biome-ignore lint/a11y/noStaticElementInteractions: SVG group is a visual click target, not a document-level interactive element
        <g
          key={key}
          data-tag={measureIndex !== undefined ? 'measure' : undefined}
          data-measure-index={measureIndex}
          onClick={
            measureIndex !== undefined
              ? () => onMeasureClick?.(measureIndex)
              : undefined
          }
          style={measureIndex !== undefined ? { cursor: 'pointer' } : undefined}
        >
          {kind.children.map((child, i) =>
            renderSvgElement(child, i, onMeasureClick),
          )}
        </g>
      )
    }
  }
}

function renderSvgDocument(
  doc: SvgDocumentOut,
  key: number,
  onMeasureClick?: (measureIndex: number) => void,
): ReactNode {
  return (
    // biome-ignore lint/a11y/noSvgWithoutTitle: synthesized score SVG; title would be redundant with surrounding page context
    <svg
      key={key}
      xmlns="http://www.w3.org/2000/svg"
      width="210mm"
      height="297mm"
      viewBox={`0 0 ${Math.round(doc.width_pt)} ${Math.round(doc.height_pt)}`}
    >
      {doc.elements.map((el, i) => renderSvgElement(el, i, onMeasureClick))}
    </svg>
  )
}

export function Preview({
  documents,
  highlightedDocuments = [],
  rendering,
  audioGenerating = false,
  wavUrl = null,
  audioAvailable = false,
  soundfontReady = false,
  onGenerateAudio,
  pdfAvailable = false,
  pdfFontsReady = false,
  pdfExporting = false,
  onExportPdf,
  splitPdfExporting = false,
  onExportSplitPdf,
  partsCount = 0,
  emptyMessage = 'No preview yet.',
  toolbar,
  onMeasureClick,
}: PreviewProps) {
  const previewPagesRef = useRef<HTMLDivElement>(null)
  const audioPlayerRef = useRef<HTMLAudioElement>(null)

  useEffect(() => {
    if (!audioGenerating) return
    const audio = audioPlayerRef.current
    if (audio && !audio.paused) {
      audio.pause()
    }
  }, [audioGenerating])

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

  const exporting = pdfExporting || splitPdfExporting
  const canExportPdf =
    pdfAvailable &&
    pdfFontsReady &&
    documents.length > 0 &&
    !rendering &&
    !exporting
  const canExportSplitPdf =
    pdfAvailable && pdfFontsReady && partsCount > 0 && !rendering && !exporting

  const activeDocs =
    highlightedDocuments.length > 0 ? highlightedDocuments : documents

  return (
    <div className="preview">
      <div className="preview-header">
        <span>Preview</span>
        <div className="preview-header-actions">
          {pdfAvailable ? (
            <button
              type="button"
              className="preview-export-btn"
              disabled={!canExportPdf}
              onClick={onExportPdf}
            >
              {pdfExporting
                ? 'Exporting PDF…'
                : !pdfFontsReady
                  ? 'Loading fonts…'
                  : 'Export PDF'}
            </button>
          ) : null}
          {pdfAvailable ? (
            <button
              type="button"
              className="preview-export-btn"
              disabled={!canExportSplitPdf}
              onClick={onExportSplitPdf}
            >
              {splitPdfExporting
                ? 'Exporting parts…'
                : !pdfFontsReady
                  ? 'Loading fonts…'
                  : 'Export parts (ZIP)'}
            </button>
          ) : null}
          {audioAvailable ? (
            <button
              type="button"
              className="preview-export-btn"
              disabled={audioGenerating || !soundfontReady}
              onClick={onGenerateAudio}
              aria-label={wavUrl ? 'Regenerate audio' : 'Generate audio'}
            >
              {audioGenerating ? (
                <>
                  <span className="preview-audio-spinner" aria-hidden="true" />
                  <span>Generating…</span>
                </>
              ) : !soundfontReady ? (
                <span>Loading soundfont…</span>
              ) : (
                <span>{wavUrl ? 'Regenerate audio' : 'Generate audio'}</span>
              )}
            </button>
          ) : null}
          {rendering ? (
            <span className="preview-status">Rendering…</span>
          ) : null}
        </div>
      </div>
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
            ref={audioPlayerRef}
            className="preview-audio-player"
            controls
            src={wavUrl}
            tabIndex={audioGenerating ? -1 : undefined}
          />
        </div>
      ) : null}
      <div className="preview-pages" ref={previewPagesRef}>
        {documents.length === 0 &&
        highlightedDocuments.length === 0 &&
        !rendering ? (
          <p className="preview-empty">{emptyMessage}</p>
        ) : null}
        {activeDocs.map((doc, i) => (
          // biome-ignore lint/suspicious/noArrayIndexKey: pages have no stable identifier
          <div key={i} className="preview-page">
            {renderSvgDocument(doc, i, onMeasureClick)}
          </div>
        ))}
      </div>
    </div>
  )
}
