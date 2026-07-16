import type { SvgDocumentOut } from 'jianpu-wasm'
import { type ReactNode, useEffect, useRef, useState } from 'react'
import { renderSvgDocument } from './PreviewSvgRenderer'
import { usePlayhead } from './usePreviewPlayhead'

interface PreviewProps {
  documents: SvgDocumentOut[]
  highlightedDocuments?: SvgDocumentOut[]
  rendering: boolean
  audioGenerating?: boolean
  wavUrl?: string | null
  wavFilename?: string
  /** Elapsed-seconds offset of each measure boundary for `wavUrl`'s audio, length = measure count + 1. */
  measureTimes?: number[]
  /** Written measure index to highlight at each playback position of `measureTimes`, following D.C. al Coda navigation. */
  writtenMeasureIndices?: number[]
  /** Elapsed-seconds offset of each measure boundary within the selected range's audio, relative to the range start. */
  measureAudioTimes?: number[]
  /** Written measure index to highlight at each playback position of `measureAudioTimes`, following D.C. al Coda navigation. */
  measureAudioWrittenIndices?: number[]
  /** The `<audio>` element currently playing the selected measure range, if any. */
  measureAudioElement?: HTMLAudioElement | null
  selectedMeasureRange?: { start: number; end: number } | null
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

export function Preview({
  documents,
  highlightedDocuments = [],
  rendering,
  audioGenerating = false,
  wavUrl = null,
  wavFilename = 'audio.wav',
  measureTimes,
  writtenMeasureIndices,
  measureAudioTimes,
  measureAudioWrittenIndices,
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

  usePlayhead(
    previewPagesRef,
    audioElement,
    measureTimes,
    0,
    writtenMeasureIndices,
  )
  usePlayhead(
    previewPagesRef,
    measureAudioElement,
    measureAudioTimes,
    selectedMeasureRange?.start ?? 0,
    measureAudioWrittenIndices,
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
