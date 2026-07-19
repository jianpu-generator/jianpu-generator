import type { NoteTimingOut, SvgDocumentOut } from 'jianpu-wasm'
import { type ReactNode, useEffect, useRef, useState } from 'react'
import { renderSvgDocument } from './PreviewSvgRenderer'
import { useNoteHighlight } from './useNoteHighlight'

interface PreviewProps {
  documents: SvgDocumentOut[]
  highlightedDocuments?: SvgDocumentOut[]
  rendering: boolean
  audioGenerating?: boolean
  wavUrl?: string | null
  wavFilename?: string
  /** Elapsed-seconds start/end of every sounding note/rest for `wavUrl`'s audio, keyed by `(source_part_index, note_id)`. */
  noteTimings?: NoteTimingOut[]
  /** Elapsed-seconds start/end of every sounding note/rest for the selected range's audio, keyed by `(source_part_index, note_id)`. */
  measureAudioNoteTimings?: NoteTimingOut[]
  /** The `<audio>` element currently playing the selected measure range, if any. */
  measureAudioElement?: HTMLAudioElement | null
  emptyMessage?: string
  toolbar?: ReactNode
  onMeasureRangeSelect?: (startIndex: number, endIndex: number) => void
  onSectionLabelClick?: (label: string) => void
}

function getSectionLabelAtPoint(x: number, y: number): string | undefined {
  const el = document.elementFromPoint(x, y)
  if (!el) return undefined
  const group = el.closest('[data-tag="section-label"]')
  if (!group) return undefined
  return (group as HTMLElement).dataset.sectionLabel
}

interface MeasureRange {
  start: number
  end: number
}

/**
 * The measure range under the given point. A merged multi-measure rest bar
 * carries a `start`/`end` wider than a single measure, so clicking anywhere
 * on it resolves to every source measure it represents.
 */
function getMeasureAtPoint(x: number, y: number): MeasureRange | undefined {
  const el = document.elementFromPoint(x, y)
  if (!el) return undefined
  const group = el.closest('[data-tag="measure"]')
  if (!group) return undefined
  const { measureIndex, measureIndexEnd } = (group as HTMLElement).dataset
  if (measureIndex === undefined) return undefined
  const start = Number.parseInt(measureIndex, 10)
  const end = Number.parseInt(measureIndexEnd ?? measureIndex, 10)
  if (Number.isNaN(start) || Number.isNaN(end)) return undefined
  return { start, end }
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

export function Preview({
  documents,
  highlightedDocuments = [],
  rendering,
  audioGenerating = false,
  wavUrl = null,
  wavFilename = 'audio.wav',
  noteTimings,
  measureAudioNoteTimings,
  measureAudioElement,
  emptyMessage = 'No preview yet.',
  toolbar,
  onMeasureRangeSelect,
  onSectionLabelClick,
}: PreviewProps) {
  const previewPagesRef = useRef<HTMLDivElement>(null)
  const [audioElement, setAudioElement] = useState<HTMLAudioElement | null>(
    null,
  )

  useNoteHighlight(previewPagesRef, audioElement, noteTimings)
  useNoteHighlight(
    previewPagesRef,
    measureAudioElement,
    measureAudioNoteTimings,
  )
  const dragStateRef = useRef<{
    anchor: MeasureRange
    current: MeasureRange
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
      const range = getMeasureAtPoint(e.clientX, e.clientY)
      if (range !== undefined) {
        dragState.current = range
        const min = Math.min(dragState.anchor.start, dragState.current.start)
        const max = Math.max(dragState.anchor.end, dragState.current.end)
        applyDragHighlights(container, min, max)
      }
    }

    const handleMouseUp = (e: MouseEvent) => {
      const dragState = dragStateRef.current
      if (!dragState) return
      const container = previewPagesRef.current
      if (container) {
        clearDragHighlights(container)
      }
      const finalRange =
        getMeasureAtPoint(e.clientX, e.clientY) ?? dragState.current
      const min = Math.min(dragState.anchor.start, finalRange.start)
      const max = Math.max(dragState.anchor.end, finalRange.end)
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
            const range = getMeasureAtPoint(e.clientX, e.clientY)
            if (range === undefined) return
            dragStateRef.current = { anchor: range, current: range }
            const container = previewPagesRef.current
            if (container) {
              applyDragHighlights(container, range.start, range.end)
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
