import { useCallback, useRef, useState } from 'react'
import { fileIdForName, selectFile } from '../fileStore'
import type { EditorHandle } from '../types'
import { useAppPanels } from './useAppPanels'
import { useAssetLoader } from './useAssetLoader'
import { useFileImport } from './useFileImport'
import { useFileOperations } from './useFileOperations'
import { useFontsLoader } from './useFontsLoader'
import { useJianpuWorker } from './useJianpuWorker'
import { useLyricSelection } from './useLyricSelection'
import { useMeasureRangeSelection } from './useMeasureRangeSelection'
import { useNoteSelection } from './useNoteSelection'
import { usePartTogglePruning } from './usePartTogglePruning'
import {
  noPartsSelected as computeNoPartsSelected,
  usePartToggles,
} from './usePartToggles'
import { useScoreSource } from './useScoreSource'
import { useSectionNavigation } from './useSectionNavigation'
import { useSequenceNavigation } from './useSequenceNavigation'
import { useStorageBackend } from './useStorageBackend'
import { useUrlFileSync } from './useUrlFileSync'
import { useWasmLoader } from './useWasmLoader'

/** Wires together every hook `App` needs — storage backend, file ops, the
 * jianpu worker, part/section/note selection, and panel state — into the one
 * flat object `App`'s JSX renders from. Split out of `App` itself to keep
 * that component under its line-count cap. */
export function useAppController() {
  const {
    store,
    setStore,
    backend,
    isLoadingGithub,
    saveStatus,
    autosaveDeadline,
    preference,
    switchBackend,
    forceSave,
    flushPendingSave,
    refreshSaveStatus,
  } = useStorageBackend()
  const [editorCollapsed, setEditorCollapsed] = useState(false)

  useUrlFileSync(store, setStore, isLoadingGithub)

  const {
    creatingFile,
    deletingFileName,
    duplicatingFile,
    renamingFileName,
    restoringFileName,
    fileOpError,
    setFileOpError,
    handleCreate,
    handleDuplicate,
    handleRename,
    handleDelete,
    handleRestore,
  } = useFileOperations(store, setStore, backend)

  const {
    sharedPreview,
    liveOwner,
    liveViewerActive,
    source,
    readOnly,
    liveShare,
  } = useScoreSource(
    store,
    backend,
    setStore,
    setFileOpError,
    setEditorCollapsed,
  )
  const fileId = fileIdForName(store, store.active)

  const editorRef = useRef<EditorHandle>(null)
  const selectedSequenceRangeRef = useRef<{
    start: number
    end: number
    entryStartIndex: number
    entryEndIndex: number
  } | null>(null)
  const soundfont = useAssetLoader('/fonts/GeneralUser_GS.sf2')
  const fonts = useFontsLoader()
  const wasm = useWasmLoader()
  const soundfontReady = soundfont.status === 'ready'
  const pdfFontsReady = fonts.status === 'ready'

  const {
    disabledParts,
    setDisabledParts,
    disabledLyrics,
    setDisabledLyrics,
    soloedParts,
    setSoloedParts,
    handlePartToggle,
    handleLyricsToggle,
    handleSoloToggle,
  } = usePartToggles(fileId)

  const {
    parts,
    partDeclarations,
    documents,
    wavUrl,
    wavFilename,
    noteTimings,
    audioAvailable,
    pdfAvailable,
    pdfExporting,
    diagnostics,
    diagnosticViewZones,
    rendering,
    audioGenerating,
    exportPdf,
    splitPdfExporting,
    exportSplitPdf,
    midiAvailable,
    midiExporting,
    exportMidi,
    splitMidiExporting,
    exportSplitMidi,
    splitWavExporting,
    exportSplitWav,
    generateFullAudio,
    selectedMeasureRange,
    measureAudioGenerating,
    measureAudioPlaying,
    measureAudioNoteTimings,
    measureAudioElement,
    measureSpans,
    sectionRanges,
    sequenceEntries,
    notifySelection,
    playSelectedMeasures,
    playFromCurrentMeasure,
    playNoteSelection,
    playAll,
    stopMeasurePlayback,
    highlightedDocuments,
    noteSpans,
    lyricSpans,
    previewInstrument,
    previewPercussion,
    stopPreviewInstrument,
    previewAudioPlaying,
    updatePartDeclaration,
    formatScore,
    shiftPartOctave,
    importFromFile,
  } = useJianpuWorker(
    source,
    disabledParts,
    disabledLyrics,
    soloedParts,
    store.active,
    soundfont.bytes,
    fonts.fonts,
    selectedSequenceRangeRef,
  )
  usePartTogglePruning(
    parts,
    setDisabledParts,
    setDisabledLyrics,
    setSoloedParts,
  )

  const handleSourceChange = useCallback(
    (value: string) => {
      setStore((prev) => backend.updateActiveContent(prev, value))
      if (liveOwner.isLive) liveOwner.broadcastContent(value)
    },
    [setStore, backend, liveOwner],
  )
  const handleSelect = useCallback(
    (name: string) => {
      flushPendingSave()
      setStore((prev) => selectFile(prev, name))
    },
    [setStore, flushPendingSave],
  )
  const handleFormatScore = useCallback(() => {
    void formatScore(source).then(handleSourceChange)
  }, [formatScore, source, handleSourceChange])
  const { importingFile, handleImportFile } = useFileImport(
    store,
    backend,
    setStore,
    setFileOpError,
    importFromFile,
  )
  const {
    editPartsOpen,
    setEditPartsOpen,
    editMetadataOpen,
    setEditMetadataOpen,
    storageSettingsOpen,
    setStorageSettingsOpen,
    binOpen,
    setBinOpen,
    handlePartDeclarationChange,
    handleShiftPartOctave,
    parsedMetadata,
    handleMetadataFieldChange,
  } = useAppPanels(
    source,
    updatePartDeclaration,
    shiftPartOctave,
    handleSourceChange,
  )

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
  } = useNoteSelection(
    noteSpans,
    parts,
    editorRef,
    measureSpans,
    notifySelection,
  )

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
    handleNoteRangeSelect,
    handleLyricRangeSelect,
    applyNoteSelectionSilently,
    applyLyricSelectionSilently,
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

  const noPartsSelected = computeNoPartsSelected(
    parts,
    disabledParts,
    soloedParts,
  )

  return {
    store,
    setStore,
    backend,
    isLoadingGithub,
    saveStatus,
    autosaveDeadline,
    preference,
    switchBackend,
    forceSave,
    refreshSaveStatus,
    editorCollapsed,
    setEditorCollapsed,
    creatingFile,
    deletingFileName,
    duplicatingFile,
    renamingFileName,
    restoringFileName,
    fileOpError,
    setFileOpError,
    handleCreate,
    handleDuplicate,
    handleRename,
    handleDelete,
    handleRestore,
    sharedPreview,
    liveViewerActive,
    source,
    readOnly,
    liveShare,
    fileId,
    editorRef,
    soundfont,
    fonts,
    wasm,
    soundfontReady,
    pdfFontsReady,
    disabledParts,
    disabledLyrics,
    soloedParts,
    handlePartToggle,
    handleLyricsToggle,
    handleSoloToggle,
    parts,
    partDeclarations,
    documents,
    wavUrl,
    wavFilename,
    noteTimings,
    audioAvailable,
    pdfAvailable,
    pdfExporting,
    diagnostics,
    diagnosticViewZones,
    rendering,
    audioGenerating,
    exportPdf,
    splitPdfExporting,
    exportSplitPdf,
    midiAvailable,
    midiExporting,
    exportMidi,
    splitMidiExporting,
    exportSplitMidi,
    splitWavExporting,
    exportSplitWav,
    generateFullAudio,
    selectedMeasureRange,
    measureAudioGenerating,
    measureAudioPlaying,
    measureAudioNoteTimings,
    measureAudioElement,
    measureSpans,
    selectedSequenceRange,
    sequenceJumpToolbarProps,
    notifySelection,
    playSelectedMeasures,
    playFromCurrentMeasure,
    playAll,
    stopMeasurePlayback,
    highlightedDocuments,
    noteSpans,
    lyricSpans,
    previewInstrument,
    previewPercussion,
    stopPreviewInstrument,
    previewAudioPlaying,
    handleSourceChange,
    handleSelect,
    handleFormatScore,
    importingFile,
    handleImportFile,
    editPartsOpen,
    setEditPartsOpen,
    editMetadataOpen,
    setEditMetadataOpen,
    storageSettingsOpen,
    setStorageSettingsOpen,
    binOpen,
    setBinOpen,
    handlePartDeclarationChange,
    handleShiftPartOctave,
    parsedMetadata,
    handleMetadataFieldChange,
    setSelectedLineRange,
    handleSectionJump,
    sectionJumpToolbarProps,
    handleNoteRangeSelect,
    handleEditorSelectionChange,
    selectedNoteRangePlaybackInfo,
    selectedNoteCells,
    handleLyricRangeSelect,
    handleLyricEditorSelectionChange,
    selectedLyricCells,
    handleMeasureRangeSelect,
    handlePlayNoteSelection,
    noPartsSelected,
  }
}
