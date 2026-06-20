import { type ReactNode, useEffect, useRef } from 'react'

interface PreviewProps {
  svgs: string[]
  highlightedSvgs?: string[]
  rendering: boolean
  audioGenerating?: boolean
  wavUrl?: string | null
  audioAvailable?: boolean
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
}

export function Preview({
  svgs,
  highlightedSvgs = [],
  rendering,
  audioGenerating = false,
  wavUrl = null,
  audioAvailable = false,
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
    if (highlightedSvgs.length === 0) return

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
  }, [highlightedSvgs])

  const exporting = pdfExporting || splitPdfExporting
  const canExportPdf =
    pdfAvailable && pdfFontsReady && svgs.length > 0 && !rendering && !exporting
  const canExportSplitPdf =
    pdfAvailable && pdfFontsReady && partsCount > 0 && !rendering && !exporting

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
              disabled={audioGenerating}
              onClick={onGenerateAudio}
              aria-label={wavUrl ? 'Regenerate audio' : 'Generate audio'}
            >
              {audioGenerating ? (
                <>
                  <span className="preview-audio-spinner" aria-hidden="true" />
                  <span>Generating…</span>
                </>
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
        {svgs.length === 0 && highlightedSvgs.length === 0 && !rendering ? (
          <p className="preview-empty">{emptyMessage}</p>
        ) : null}
        {(highlightedSvgs.length > 0 ? highlightedSvgs : svgs).map((svg) => (
          <div
            key={svg}
            className="preview-page"
            // biome-ignore lint/security/noDangerouslySetInnerHtml: trusted SVG from local WASM renderer
            dangerouslySetInnerHTML={{ __html: svg }}
          />
        ))}
      </div>
    </div>
  )
}
