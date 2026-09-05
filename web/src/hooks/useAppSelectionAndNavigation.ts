import type { RefObject } from 'react'
import { useCallback, useMemo } from 'react'
import type {
  EditorHandle,
  LyricSpan,
  MeasureSpan,
  NoteSpan,
  PartInfo,
  SectionRange,
  SequenceEntry,
} from '../types'
import type { NoteCell } from '../utils/noteSpanSelection'
import { useLyricSelection } from './useLyricSelection'
import { useMeasureRangeSelection } from './useMeasureRangeSelection'
import { useNoteSelection } from './useNoteSelection'
import { useSectionNavigation } from './useSectionNavigation'
import { useSequenceNavigation } from './useSequenceNavigation'

/** Wires together section/sequence jump navigation and note/lyric/measure
 * range selection — the editor-selection half of `useAppController`. Split
 * out to keep `useAppController` under its line-count cap. */
export function useAppSelectionAndNavigation(
  sectionRanges: SectionRange[],
  editorRef: RefObject<EditorHandle | null>,
  notifySelection: (
    firstLine: number,
    lastLine: number,
    isEmpty: boolean,
    revealLine?: number,
    measureRanges?: { start: number; end: number }[],
  ) => void,
  sequenceEntries: SequenceEntry[],
  measureSpans: MeasureSpan[],
  selectedSequenceRangeRef: RefObject<{
    start: number
    end: number
    entryStartIndex: number
    entryEndIndex: number
  } | null>,
  noteSpans: NoteSpan[],
  parts: PartInfo[],
  enabledTracks: string[] | undefined,
  lyricSpans: LyricSpan[],
  playNoteSelection: (
    minMeasureIndex: number,
    maxMeasureIndex: number,
    selectedPartNames: string[],
    selectedCells: NoteCell[],
  ) => void,
) {
  const {
    handleNoteRangeSelect,
    handleEditorSelectionChange,
    selectedNoteRangePlaybackInfo,
    selectedNoteCells: noteSelectionCells,
    applyNoteSelectionSilently,
    clearNoteSelection,
  } = useNoteSelection(noteSpans, parts, enabledTracks, editorRef)

  const {
    handleLyricRangeSelect,
    handleEditorSelectionChange: handleLyricEditorSelectionChange,
    selectedLyricCells: lyricSelectionCells,
    applyLyricSelectionSilently,
    clearLyricSelection,
  } = useLyricSelection(lyricSpans, editorRef)

  const {
    handleMeasureRangeSelect,
    measureRangeNoteCells,
    measureRangeLyricCells,
    clearMeasureRangeSelection,
  } = useMeasureRangeSelection(
    editorRef,
    noteSpans,
    lyricSpans,
    applyNoteSelectionSilently,
    applyLyricSelectionSilently,
    measureSpans,
    notifySelection,
  )

  // Fed to `useSectionNavigation`/`useSequenceNavigation` below — a section
  // or sequence jump replaces whatever a prior no-mounted-editor (Live/
  // shared view) note/lyric tap or measure/bar-line click left painted,
  // rather than layering on top of it (see those hooks' own
  // `clearNoMountedEditorHighlights` param doc comment).
  const clearNoMountedEditorHighlights = useCallback(() => {
    clearNoteSelection()
    clearLyricSelection()
    clearMeasureRangeSelection()
  }, [clearNoteSelection, clearLyricSelection, clearMeasureRangeSelection])

  const { setSelectedLineRange, handleSectionJump, sectionJumpToolbarProps } =
    useSectionNavigation(
      sectionRanges,
      editorRef,
      measureSpans,
      notifySelection,
      clearNoMountedEditorHighlights,
    )

  const { selectedSequenceRange, sequenceJumpToolbarProps } =
    useSequenceNavigation(
      sequenceEntries,
      measureSpans,
      editorRef,
      notifySelection,
      selectedSequenceRangeRef,
      clearNoMountedEditorHighlights,
    )

  // Merged purely for `Preview.tsx`'s highlight painting: an editor-mounted
  // drag/click populates `noteSelectionCells`/`lyricSelectionCells` and
  // leaves `measureRangeNoteCells`/`measureRangeLyricCells` at `[]`; a
  // no-mounted-editor (Live/shared) measure/bar-line gesture does the
  // opposite (see `useMeasureRangeSelection`'s doc comment) — the two never
  // hold cells at the same time, so concatenating is a safe union, not an
  // accidental widening of either state's own meaning.
  const selectedNoteCells = useMemo(
    () => [...noteSelectionCells, ...measureRangeNoteCells],
    [noteSelectionCells, measureRangeNoteCells],
  )
  const selectedLyricCells = useMemo(
    () => [...lyricSelectionCells, ...measureRangeLyricCells],
    [lyricSelectionCells, measureRangeLyricCells],
  )

  const handlePlayNoteSelection = useCallback(() => {
    if (selectedNoteRangePlaybackInfo === null) return
    playNoteSelection(
      selectedNoteRangePlaybackInfo.minMeasureIndex,
      selectedNoteRangePlaybackInfo.maxMeasureIndex,
      selectedNoteRangePlaybackInfo.selectedPartNames,
      noteSelectionCells,
    )
  }, [selectedNoteRangePlaybackInfo, noteSelectionCells, playNoteSelection])

  return {
    setSelectedLineRange,
    handleSectionJump,
    sectionJumpToolbarProps,
    selectedSequenceRange,
    sequenceJumpToolbarProps,
    handleNoteRangeSelect,
    handleEditorSelectionChange,
    selectedNoteRangePlaybackInfo,
    selectedNoteCells,
    handleLyricRangeSelect,
    handleLyricEditorSelectionChange,
    selectedLyricCells,
    handleMeasureRangeSelect,
    handlePlayNoteSelection,
  }
}
