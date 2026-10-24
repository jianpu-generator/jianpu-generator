import { useCallback, useRef, useState } from 'react'
import { fileIdForName, selectFile } from '../fileStore'
import type { EditorHandle } from '../types'
import { useAppPanels } from './useAppPanels'
import { useAppSelectionAndNavigation } from './useAppSelectionAndNavigation'
import { useAssetLoader } from './useAssetLoader'
import { useDocumentTitle } from './useDocumentTitle'
import { useFileImport } from './useFileImport'
import { useFileOperations } from './useFileOperations'
import { useFontsLoader } from './useFontsLoader'
import { useJianpuWorker } from './useJianpuWorker'
import { usePartTogglePruning } from './usePartTogglePruning'
import { usePartToggles } from './usePartToggles'
import { useScoreSource } from './useScoreSource'
import type { SelectedSequenceRange } from './useSequenceNavigation'
import { useStorageBackend } from './useStorageBackend'
import { useUrlFileSync } from './useUrlFileSync'
import { useWasmLoader } from './useWasmLoader'
import { noPartsSelected as computeNoPartsSelected } from './workerHelpers'

/** Wires every hook `App` needs into the one flat object its JSX renders
 * from. Split out of `App` to keep it under the line-count cap. */
export function useAppController() {
  const {
    store,
    setStore,
    backend,
    isLoadingCloud,
    saveStatus,
    saveLabel,
    autosaveDeadline,
    preference,
    switchBackend,
    forceSave,
    flushPendingSave,
    refreshSaveStatus,
  } = useStorageBackend()
  const [editorCollapsed, setEditorCollapsed] = useState(false)
  useUrlFileSync(store, setStore, isLoadingCloud)

  const {
    creatingFile,
    creatingFolder,
    deletingFileName,
    duplicatingFile,
    renamingFileName,
    restoringFileName,
    fileOpError,
    setFileOpError,
    handleCreate,
    handleCreateFolder,
    handleDuplicate,
    handleRename,
    handleDelete,
    handleMoveFolder,
    handleDeleteFolder,
    handleMoveSelection,
    handleDeleteSelection,
    handleRestore,
  } = useFileOperations(store, setStore, backend)
  const {
    sharedPreview,
    sharedFolder,
    syncedShareViewerActive,
    source,
    readOnly,
    syncedShare,
  } = useScoreSource(
    store,
    backend,
    setStore,
    setFileOpError,
    setEditorCollapsed,
    saveStatus,
  )
  const fileId = sharedFolder?.fileId ?? fileIdForName(store, store.active)
  useDocumentTitle(store.active)
  const editorRef = useRef<EditorHandle>(null)
  const selectedSequenceRangeRef = useRef<SelectedSequenceRange | null>(null)
  const measureRangeSelectedPartNamesRef = useRef<string[] | undefined>(
    undefined,
  )
  const soundfont = useAssetLoader('/fonts/GeneralUser_GS.sf2')
  const fonts = useFontsLoader()
  const wasm = useWasmLoader()
  const soundfontReady = soundfont.status === 'ready'
  const pdfFontsReady = fonts.status === 'ready'

  const {
    partToggles,
    disabledParts,
    setDisabledParts,
    soloedParts,
    setSoloedParts,
    handlePartToggle,
    handleSoloToggle,
  } = usePartToggles(fileId)

  const {
    parts,
    visibility,
    partDeclarations,
    documents,
    pendingDownload,
    requestDownload,
    confirmPendingDownload,
    cancelPendingDownload,
    wavUrl,
    wavFilename,
    mp3Url,
    mp3Filename,
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
    mp3Available,
    mp3Exporting,
    exportMp3,
    splitMp3Exporting,
    exportSplitMp3,
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
    metronomeEnabled,
    setMetronomeEnabled,
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
    editRange,
    describeSelection,
    importFromFile,
    staffNotation,
  } = useJianpuWorker(
    source,
    partToggles,
    wasm.status === 'ready',
    store.active,
    soundfont.bytes,
    fonts.fonts,
    selectedSequenceRangeRef,
    measureRangeSelectedPartNamesRef,
  )
  usePartTogglePruning(parts, setDisabledParts, setSoloedParts)

  const handleSourceChange = useCallback(
    (value: string) => {
      setStore((prev) => backend.updateActiveContent(prev, value))
    },
    [setStore, backend],
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
  const panels = useAppPanels(
    source,
    updatePartDeclaration,
    shiftPartOctave,
    handleSourceChange,
  )

  const {
    setSelectedLineRange,
    handleSectionJump,
    sectionJumpToolbarProps,
    selectedSequenceRange,
    sequenceJumpToolbarProps,
    handleNoteRangeSelect,
    handleEditorSelectionChange,
    selectedNoteRangePlaybackInfo,
    selectedNoteCells,
    selectedNoteByteRanges,
    handleLyricRangeSelect,
    handleLyricEditorSelectionChange,
    selectedLyricCells,
    handleMeasureRangeSelect,
    handlePlayNoteSelection,
    handleEditSelection,
  } = useAppSelectionAndNavigation(
    sectionRanges,
    editorRef,
    notifySelection,
    sequenceEntries,
    measureSpans,
    selectedSequenceRangeRef,
    noteSpans,
    lyricSpans,
    playNoteSelection,
    editRange,
    handleSourceChange,
    measureRangeSelectedPartNamesRef,
  )

  const noPartsSelected = computeNoPartsSelected(parts, visibility)
  return {
    store,
    setStore,
    backend,
    isLoadingCloud,
    saveStatus,
    saveLabel,
    autosaveDeadline,
    preference,
    switchBackend,
    forceSave,
    refreshSaveStatus,
    editorCollapsed,
    setEditorCollapsed,
    creatingFile,
    creatingFolder,
    deletingFileName,
    duplicatingFile,
    renamingFileName,
    restoringFileName,
    fileOpError,
    setFileOpError,
    handleCreate,
    handleCreateFolder,
    handleDuplicate,
    handleRename,
    handleDelete,
    handleMoveFolder,
    handleDeleteFolder,
    handleMoveSelection,
    handleDeleteSelection,
    handleRestore,
    sharedPreview,
    syncedShareViewerActive,
    source,
    readOnly,
    syncedShare,
    fileId,
    editorRef,
    soundfont,
    fonts,
    wasm,
    soundfontReady,
    pdfFontsReady,
    disabledParts,
    soloedParts,
    partToggles,
    handlePartToggle,
    handleSoloToggle,
    parts,
    partDeclarations,
    documents,
    pendingDownload,
    requestDownload,
    confirmPendingDownload,
    cancelPendingDownload,
    wavUrl,
    wavFilename,
    mp3Url,
    mp3Filename,
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
    mp3Available,
    mp3Exporting,
    exportMp3,
    splitMp3Exporting,
    exportSplitMp3,
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
    metronomeEnabled,
    setMetronomeEnabled,
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
    handleEditSelection,
    describeSelection,
    importingFile,
    handleImportFile,
    panels,
    setSelectedLineRange,
    handleSectionJump,
    sectionJumpToolbarProps,
    handleNoteRangeSelect,
    handleEditorSelectionChange,
    selectedNoteRangePlaybackInfo,
    selectedNoteCells,
    selectedNoteByteRanges,
    handleLyricRangeSelect,
    handleLyricEditorSelectionChange,
    selectedLyricCells,
    handleMeasureRangeSelect,
    handlePlayNoteSelection,
    noPartsSelected,
    staffNotation,
  }
}
