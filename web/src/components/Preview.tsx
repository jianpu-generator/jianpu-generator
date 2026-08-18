import type { NoteTimingOut, SvgDocumentOut } from 'jianpu-wasm'
import { useEffect, useRef, useState } from 'react'
import type { LyricSpan, NoteSpan } from '../types'
import { renderSvgDocument } from './PreviewSvgRenderer'
import {
  applyPartLabelDragHighlight,
  applyPersistedLyricHighlights,
  applyPersistedNoteHighlights,
  applyPersistedPartLabelHighlights,
} from './previewDragHighlights'
import {
  getLyricAtPoint,
  getMeasureAtPoint,
  getNoteAtPoint,
  getPartLabelAtPoint,
  getSectionLabelAtPoint,
  type LyricCell,
  lyricCellsInMeasureRange,
  type NoteCell,
  noteCellsForPartLabels,
  noteCellsInMeasureRange,
} from './previewSelection'
import { usePlaybackCursor } from './usePlaybackCursor'
import { usePreviewDragSelection } from './usePreviewDragSelection'

export type { LyricCell, NoteCell } from './previewSelection'

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
  onSectionLabelClick?: (label: string) => void
  /** Fired on mouseup after a note-level drag-select (see `getNoteAtPoint`),
   * with every note/rest cell the drag's marquee overlapped — but only when
   * `onMeasureRangeSelect` isn't supplied (see `onLyricRangeSelect` below for
   * why: a note drag's marquee can also cover lyric syllables underneath
   * it, which then routes through `onMeasureRangeSelect` instead). */
  onNoteRangeSelect?: (selectedCells: NoteCell[]) => void
  /** The note/rest cells from the most recent note drag-select (see
   * `onNoteRangeSelect`), echoed back so the highlight can be re-applied
   * declaratively — including after a re-render swaps in fresh SVG DOM
   * (e.g. the Monaco selection this drag pushed triggering a highlighted
   * re-render), which would otherwise silently drop the highlight. */
  selectedNoteCells?: NoteCell[]
  /** Per-note/rest `(source_part_index, note_id) → measure_index` mapping,
   * used by `noteCellsInMeasureRange` to resolve a measure click/drag into
   * every note cell it contains by index rather than pixel geometry. */
  noteSpans?: NoteSpan[]
  /** Fired on mouseup after a lyric-syllable drag-select (see
   * `getLyricAtPoint`), with every syllable cell the drag's marquee
   * overlapped — but only when `onMeasureRangeSelect` isn't supplied. A
   * lyric drag's marquee can also visually cover notes above it (and vice
   * versa for a note drag), so whenever `onMeasureRangeSelect` is wired up
   * it's used instead — with both the note cells and the lyric cells the
   * marquee overlapped, empty array or not — since a mounted editor's Monaco
   * selection can only take one combined push per gesture rather than one
   * from each of this and `onNoteRangeSelect` independently (see
   * `usePreviewDragSelection`'s mouseup handler and `onMeasureRangeSelect`
   * below). */
  onLyricRangeSelect?: (selectedCells: LyricCell[]) => void
  /** The lyric syllable cells from the most recent lyric drag-select (see
   * `onLyricRangeSelect`), echoed back so the highlight can be re-applied
   * declaratively, mirroring `selectedNoteCells`. */
  selectedLyricCells?: LyricCell[]
  /** Per-lyric-syllable `(source_part_index, note_id, verse) → measure_index`
   * mapping, used by `lyricCellsInMeasureRange` so a measure click/drag also
   * selects the verse lyrics under it, alongside `noteSpans` for notes. */
  lyricSpans?: LyricSpan[]
  /** Fired instead of `onNoteRangeSelect`/`onLyricRangeSelect` for a
   * measure/bar-line click or drag, and also for a note- or lyric-level
   * drag/click whose marquee visually covers the other cell type too (see
   * `usePreviewDragSelection`), with every note cell and every lyric cell
   * the gesture resolved to. Any of the three gestures can select both cell
   * types at once, so this is a single combined callback rather than one
   * call to each of the other two — see `useAppController`'s
   * `handleMeasureRangeSelect` for why calling both independently doesn't
   * work. */
  onMeasureRangeSelect?: (
    noteCells: NoteCell[],
    lyricCells: LyricCell[],
  ) => void
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
  onSectionLabelClick,
  onNoteRangeSelect,
  selectedNoteCells = [],
  noteSpans = [],
  onLyricRangeSelect,
  selectedLyricCells = [],
  lyricSpans = [],
  onMeasureRangeSelect,
}: PreviewProps) {
  const previewPagesRef = useRef<HTMLDivElement>(null)
  const [audioElement, setAudioElement] = useState<HTMLAudioElement | null>(
    null,
  )
  const noteSpansRef = useRef(noteSpans)
  noteSpansRef.current = noteSpans

  usePlaybackCursor(previewPagesRef, audioElement, noteTimings)
  usePlaybackCursor(
    previewPagesRef,
    measureAudioElement,
    measureAudioNoteTimings,
  )
  const dragStateRef = usePreviewDragSelection(
    previewPagesRef,
    noteSpans,
    onNoteRangeSelect,
    onLyricRangeSelect,
    lyricSpans,
    onMeasureRangeSelect,
  )
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

  // Re-applies the note drag-select highlight declaratively from
  // `selectedNoteCells` on every relevant render, rather than leaving it as
  // a one-shot imperative toggle on mouseup — a re-render can swap in fresh
  // SVG DOM (e.g. `documents`/`highlightedDocuments` changing after the
  // Monaco selection this drag pushed), which would silently wipe any
  // dataset attribute set only during the drag itself.
  // biome-ignore lint/correctness/useExhaustiveDependencies: documents/highlightedDocuments aren't read in the body, but must stay listed so this re-runs after they swap in fresh SVG DOM (see comment above).
  useEffect(() => {
    const container = previewPagesRef.current
    if (!container) return
    applyPersistedNoteHighlights(container, selectedNoteCells)
    applyPersistedPartLabelHighlights(
      container,
      noteSpansRef.current,
      selectedNoteCells,
    )
  }, [selectedNoteCells, documents, highlightedDocuments])

  // Mirrors the effect above for lyric syllable selection — independent of
  // `selectedNoteCells`, since a lyric selection never drives note
  // highlighting and vice versa (see `useLyricSelection`).
  // biome-ignore lint/correctness/useExhaustiveDependencies: documents/highlightedDocuments aren't read in the body, but must stay listed so this re-runs after they swap in fresh SVG DOM (see comment above).
  useEffect(() => {
    const container = previewPagesRef.current
    if (!container) return
    applyPersistedLyricHighlights(container, selectedLyricCells)
  }, [selectedLyricCells, documents, highlightedDocuments])

  const activeDocs =
    highlightedDocuments.length > 0 ? highlightedDocuments : documents

  return (
    <div className="preview">
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
        {/* biome-ignore lint/a11y/noStaticElementInteractions: drag/click-to-select notes uses mousedown, mousemove, mouseup — not a standard interactive role */}
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
            const partLabel = getPartLabelAtPoint(e.clientX, e.clientY)
            if (partLabel !== undefined) {
              const point = { x: e.clientX, y: e.clientY }
              dragStateRef.current = {
                mode: 'part-label',
                anchor: point,
                current: point,
                anchorSystem: {
                  measureIndexStart: partLabel.measureIndexStart,
                  measureIndexEnd: partLabel.measureIndexEnd,
                },
              }
              const container = previewPagesRef.current
              if (container) {
                applyPartLabelDragHighlight(container, [partLabel])
                applyPersistedNoteHighlights(
                  container,
                  noteCellsForPartLabels(noteSpans, [partLabel]),
                )
              }
              e.preventDefault()
              return
            }
            // Checked before the note click-target below: a lyric
            // syllable's own click target paints on top of (and never
            // overlaps outside of) the note's wider click-target rect, so a
            // hit here means the click landed on the syllable's own rect —
            // see `Tag::Lyric`'s doc comment and
            // `resolve_click_target_elements`'s append order.
            const lyricCell = getLyricAtPoint(e.clientX, e.clientY)
            if (lyricCell !== undefined) {
              const point = { x: e.clientX, y: e.clientY }
              dragStateRef.current = {
                mode: 'lyric',
                anchor: point,
                current: point,
              }
              const container = previewPagesRef.current
              if (container) {
                applyPersistedLyricHighlights(container, [lyricCell])
              }
              e.preventDefault()
              return
            }
            const noteCell = getNoteAtPoint(e.clientX, e.clientY)
            if (noteCell !== undefined) {
              const point = { x: e.clientX, y: e.clientY }
              const measureRangeAtAnchor = getMeasureAtPoint(
                e.clientX,
                e.clientY,
              )
              dragStateRef.current = {
                mode: 'pending',
                anchor: point,
                noteCellAtAnchor: noteCell,
                measureRangeAtAnchor,
              }
              // Eagerly show the whole-measure highlight, matching a plain
              // click's instant-highlight-on-mousedown — overwritten if this
              // turns into a real note-drag (see `usePreviewDragSelection`).
              const container = previewPagesRef.current
              if (container && measureRangeAtAnchor !== undefined) {
                applyPersistedNoteHighlights(
                  container,
                  noteCellsInMeasureRange(noteSpans, measureRangeAtAnchor),
                )
                applyPersistedLyricHighlights(
                  container,
                  lyricCellsInMeasureRange(lyricSpans, measureRangeAtAnchor),
                )
              }
              e.preventDefault()
              return
            }
            const range = getMeasureAtPoint(e.clientX, e.clientY)
            if (range === undefined) return
            dragStateRef.current = {
              mode: 'measure',
              anchor: range,
              current: range,
            }
            const container = previewPagesRef.current
            if (container) {
              applyPersistedNoteHighlights(
                container,
                noteCellsInMeasureRange(noteSpans, range),
              )
              applyPersistedLyricHighlights(
                container,
                lyricCellsInMeasureRange(lyricSpans, range),
              )
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
