import type { NoteTimingOut, SvgDocumentOut } from 'jianpu-wasm'
import { useEffect, useRef, useState } from 'react'
import type { LyricSpan, NoteSpan } from '../types'
import { renderSvgDocument } from './PreviewSvgRenderer'
import {
  applyPersistedLyricHighlights,
  applyPersistedNoteHighlights,
} from './previewDragHighlights'
import {
  applyPersistedLyricLabelHighlights,
  applyPersistedPartLabelHighlights,
} from './previewLabelDragHighlights'
import { handlePreviewMouseDown } from './previewMouseDownHandler'
import type { LyricCell, NoteCell } from './previewSelection'
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
  /** Mirrors `audioGenerating`/`wavUrl`/`wavFilename` for the MP3 format —
   * mutually exclusive with the WAV trio (only one of `wavUrl`/`mp3Url` is
   * ever non-null), so the inline player below always shows whichever
   * format was most recently generated. */
  mp3Exporting?: boolean
  mp3Url?: string | null
  mp3Filename?: string
  /** Elapsed-seconds start/end of every sounding note/rest for whichever of `wavUrl`/`mp3Url` is set, keyed by `(source_part_index, note_id)`. */
  noteTimings?: NoteTimingOut[]
  /** Elapsed-seconds start/end of every sounding note/rest for the selected range's audio, keyed by `(source_part_index, note_id)`. */
  measureAudioNoteTimings?: NoteTimingOut[]
  /** The `<audio>` element currently playing the selected measure range, if any. */
  measureAudioElement?: HTMLAudioElement | null
  emptyMessage?: string
  /** Opens the rename-before-download modal for the inline audio player's
   * "Download" button, instead of downloading `audioUrl` immediately —
   * see `PendingDownload` in `useJianpuWorkerTypes.ts`. */
  onRequestAudioDownload?: (url: string, filename: string) => void
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
  /** The measure range backing the current selection (caret or range),
   * regardless of whether it's caret-only — used to scroll the preview to
   * the selection even when `highlightedDocuments` isn't populated (e.g. a
   * section/sequence jump, which selects a real range and therefore opts
   * out of the amber caret-only highlight). See `selectedNoteCells` above
   * for the note/lyric-level analogue. `revealMeasureIndex` is which
   * measure to scroll to, when it isn't `start` — a section/sequence chain
   * selection can resolve to a written-measure range whose document-order
   * start isn't where the user actually navigated to. */
  selectedMeasureRange?: {
    start: number
    end: number
    revealMeasureIndex: number
    /** The exact disjoint measure ranges highlighted in the SVG preview for
     * this selection (a `# sequence` chain), when it differs from the
     * single `[start, end]` span above. Not read directly by `Preview`
     * itself — the highlight rects it drives come back through
     * `highlightedDocuments`, already rendered — but kept here so this
     * duplicated local type matches `selectedMeasureRange`'s shape
     * everywhere else it's declared. */
    highlightRanges?: { start: number; end: number }[]
  } | null
}

export function Preview({
  documents,
  highlightedDocuments = [],
  rendering,
  audioGenerating = false,
  wavUrl = null,
  wavFilename = 'audio.wav',
  mp3Exporting = false,
  mp3Url = null,
  mp3Filename = 'audio.mp3',
  noteTimings,
  measureAudioNoteTimings,
  measureAudioElement,
  emptyMessage = 'No preview yet.',
  onRequestAudioDownload,
  onSectionLabelClick,
  onNoteRangeSelect,
  selectedNoteCells = [],
  noteSpans = [],
  onLyricRangeSelect,
  selectedLyricCells = [],
  lyricSpans = [],
  onMeasureRangeSelect,
  selectedMeasureRange = null,
}: PreviewProps) {
  const previewPagesRef = useRef<HTMLDivElement>(null)
  const [audioElement, setAudioElement] = useState<HTMLAudioElement | null>(
    null,
  )
  const noteSpansRef = useRef(noteSpans)
  noteSpansRef.current = noteSpans
  const lyricSpansRef = useRef(lyricSpans)
  lyricSpansRef.current = lyricSpans

  // wavUrl/mp3Url are mutually exclusive (see `useJianpuWorkerAudioActions`),
  // so whichever is set is the one inline player below renders. noteTimings
  // applies to whichever format that is — the worker's `audio`/`mp3`
  // responses each pair their bytes with a matching `noteTimings` computed
  // straight from source (codec-independent), so the cursor animates the
  // same way regardless of format.
  const audioUrl = wavUrl ?? mp3Url
  const audioFilename = wavUrl ? wavFilename : mp3Filename
  const audioBusy = wavUrl ? audioGenerating : mp3Exporting
  const audioNoteTimings = audioUrl ? noteTimings : undefined

  usePlaybackCursor(previewPagesRef, audioElement, audioNoteTimings)
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
    if (!audioBusy) return
    if (audioElement && !audioElement.paused) {
      audioElement.pause()
    }
  }, [audioBusy, audioElement])

  // Scrolls the preview to the current selection. Prefers the amber
  // caret-only highlight rect (present only when `highlightedDocuments` is
  // populated) so the highlighted measure lands dead center; otherwise
  // falls back to the plain document's own `[data-tag="measure"]` group for
  // `selectedMeasureRange.revealMeasureIndex`, which exists regardless of
  // highlight state — covering range selections (e.g. section/sequence
  // jumps) that deliberately opt out of the caret-only highlight but still
  // need the preview to scroll to them. `revealMeasureIndex` matters
  // because a chain selection's own `start` is wherever the chain's
  // earliest entry sits in document order, not necessarily where the user
  // navigated to (see `measureRangeInSpanWithReveal`).
  // biome-ignore lint/correctness/useExhaustiveDependencies: documents/highlightedDocuments aren't read in the body, but must stay listed so this re-runs after they swap in fresh SVG DOM (see comment above effect near the top of this file).
  useEffect(() => {
    if (selectedMeasureRange === null) return
    const targetMeasureIndex = selectedMeasureRange.revealMeasureIndex

    const frameId = requestAnimationFrame(() => {
      const container = previewPagesRef.current
      if (!container) return

      const target =
        container.querySelector('[data-testid="measure-highlight"]') ??
        container.querySelector(
          `[data-tag="measure"][data-measure-index="${targetMeasureIndex}"]`,
        )
      target?.scrollIntoView({
        block: 'center',
        inline: 'nearest',
      })
    })

    return () => cancelAnimationFrame(frameId)
  }, [selectedMeasureRange, documents, highlightedDocuments])

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
    applyPersistedLyricLabelHighlights(
      container,
      lyricSpansRef.current,
      selectedLyricCells,
    )
  }, [selectedLyricCells, documents, highlightedDocuments])

  const activeDocs =
    highlightedDocuments.length > 0 ? highlightedDocuments : documents

  return (
    <div className="preview">
      {audioUrl ? (
        <div
          className={
            audioBusy
              ? 'preview-audio preview-audio--generating'
              : 'preview-audio'
          }
          aria-busy={audioBusy || undefined}
        >
          {/* biome-ignore lint/a11y/useMediaCaption: synthesized score preview has no captions track */}
          <audio
            ref={setAudioElement}
            className="preview-audio-player"
            controls
            src={audioUrl}
            tabIndex={audioBusy ? -1 : undefined}
          />
          <button
            type="button"
            className="preview-audio-download"
            data-testid="preview-audio-download-button"
            onClick={() => onRequestAudioDownload?.(audioUrl, audioFilename)}
            tabIndex={audioBusy ? -1 : undefined}
          >
            Download
          </button>
        </div>
      ) : null}
      <div className="preview-pages-wrapper">
        {/* biome-ignore lint/a11y/noStaticElementInteractions: drag/click-to-select notes uses mousedown, mousemove, mouseup — not a standard interactive role */}
        <div
          className="preview-pages"
          ref={previewPagesRef}
          onMouseDown={(e) =>
            handlePreviewMouseDown(e, {
              dragStateRef,
              previewPagesRef,
              noteSpans,
              lyricSpans,
              onSectionLabelClick: onSectionLabelClickRef.current,
            })
          }
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
