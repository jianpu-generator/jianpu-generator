import type { RefObject } from 'react'
import { useCallback } from 'react'
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
  const { setSelectedLineRange, handleSectionJump, sectionJumpToolbarProps } =
    useSectionNavigation(sectionRanges, editorRef, notifySelection)

  const { selectedSequenceRange, sequenceJumpToolbarProps } =
    useSequenceNavigation(
      sequenceEntries,
      measureSpans,
      editorRef,
      notifySelection,
      selectedSequenceRangeRef,
    )

  const {
    handleNoteRangeSelect,
    handleEditorSelectionChange,
    selectedNoteRangePlaybackInfo,
    selectedNoteCells,
    applyNoteSelectionSilently,
  } = useNoteSelection(noteSpans, parts, enabledTracks, editorRef)

  const {
    handleLyricRangeSelect,
    handleEditorSelectionChange: handleLyricEditorSelectionChange,
    selectedLyricCells,
    applyLyricSelectionSilently,
  } = useLyricSelection(lyricSpans, editorRef)

  const handleMeasureRangeSelect = useMeasureRangeSelection(
    editorRef,
    noteSpans,
    lyricSpans,
    applyNoteSelectionSilently,
    applyLyricSelectionSilently,
    measureSpans,
    notifySelection,
  )

  const handlePlayNoteSelection = useCallback(() => {
    if (selectedNoteRangePlaybackInfo === null) return
    playNoteSelection(
      selectedNoteRangePlaybackInfo.minMeasureIndex,
      selectedNoteRangePlaybackInfo.maxMeasureIndex,
      selectedNoteRangePlaybackInfo.selectedPartNames,
      selectedNoteCells,
    )
  }, [selectedNoteRangePlaybackInfo, selectedNoteCells, playNoteSelection])

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
