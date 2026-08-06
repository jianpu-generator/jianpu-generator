import { useCallback, useRef, useState } from 'react'
import { AppHeader } from './components/AppHeader'
import { AppOverlays } from './components/AppOverlays'
import { AppWorkspace } from './components/AppWorkspace'
import { AssetLoadingBanner } from './components/AssetLoadingBanner'
import { ExportAudioToast } from './components/ExportAudioToast'
import { PartToggles } from './components/PartToggles'
import { SectionJumpToolbar } from './components/SectionJumpToolbar'
import { SequenceJumpToolbar } from './components/SequenceJumpToolbar'
import { fileIdForName, selectFile } from './fileStore'
import { useAppPanels } from './hooks/useAppPanels'
import { useAssetLoader } from './hooks/useAssetLoader'
import { useFileImport } from './hooks/useFileImport'
import { useFileOperations } from './hooks/useFileOperations'
import { useFontsLoader } from './hooks/useFontsLoader'
import { useJianpuWorker } from './hooks/useJianpuWorker'
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts'
import { usePartTogglePruning } from './hooks/usePartTogglePruning'
import {
  noPartsSelected as computeNoPartsSelected,
  usePartToggles,
} from './hooks/usePartToggles'
import { useScoreSource } from './hooks/useScoreSource'
import { useSectionNavigation } from './hooks/useSectionNavigation'
import { useStorageBackend } from './hooks/useStorageBackend'
import { useUnzippedViewState } from './hooks/useUnzippedViewState'
import { useUnzippedViewToggle } from './hooks/useUnzippedViewToggle'
import { useUrlFileSync } from './hooks/useUrlFileSync'
import { useWasmLoader } from './hooks/useWasmLoader'
import type { EditorHandle } from './types'
import {
  playFromCurrentMeasureShortcutLabel,
  shortcutLabel,
} from './utils/shortcutLabels'
import './App.css'
import './file-switcher.css'
import './preview.css'

export default function App() {
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
  const [unzippedView, setUnzippedView] = useUnzippedViewState(fileId)

  const editorRef = useRef<EditorHandle>(null)
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
    selectedSequenceRange,
    sequenceJumpToolbarProps,
    notifySelection,
    notifyUnzippedSelection,
    unzippedText,
    partMeasureRanges,
    playSelectedMeasures,
    playFromCurrentMeasure,
    stopMeasurePlayback,
    highlightedDocuments,
    previewInstrument,
    previewPercussion,
    stopPreviewInstrument,
    previewAudioPlaying,
    updatePartDeclaration,
    formatScore,
    formatUnzippedText,
    importFromFile,
  } = useJianpuWorker(
    source,
    disabledParts,
    disabledLyrics,
    soloedParts,
    store.active,
    soundfont.bytes,
    fonts.fonts,
    unzippedView,
    editorRef,
  )
  usePartTogglePruning(
    parts,
    setDisabledParts,
    setDisabledLyrics,
    setSoloedParts,
  )
  useKeyboardShortcuts({
    measureAudioPlaying,
    measureAudioGenerating,
    soundfontReady,
    selectedMeasureRange,
    selectedSequenceRange,
    playSelectedMeasures,
    playFromCurrentMeasure,
    stopMeasurePlayback,
    forceSave,
  })

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
  const { handleFormatScore, handleToggleUnzippedView } = useUnzippedViewToggle(
    {
      unzippedView,
      setUnzippedView,
      formatScore,
      source,
      handleSourceChange,
    },
  )
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
    parsedMetadata,
    handleMetadataFieldChange,
  } = useAppPanels(source, updatePartDeclaration, handleSourceChange)

  const {
    setSelectedLineRange,
    handleSectionJump,
    handleMeasureRangeSelect,
    sectionJumpToolbarProps,
  } = useSectionNavigation(
    sectionRanges,
    measureSpans,
    editorRef,
    notifySelection,
  )

  const noPartsSelected = computeNoPartsSelected(
    parts,
    disabledParts,
    soloedParts,
  )

  return (
    <div className="app">
      <AssetLoadingBanner
        soundfontStatus={soundfont.status}
        soundfontLoadedBytes={soundfont.loadedBytes}
        soundfontTotalBytes={soundfont.totalBytes}
        fontsStatus={fonts.status}
        fontsLoadedBytes={fonts.loadedBytes}
        fontsTotalBytes={fonts.totalBytes}
        wasmStatus={wasm.status}
        wasmLoadedBytes={wasm.loadedBytes}
        wasmTotalBytes={wasm.totalBytes}
      />
      <AppHeader
        audioAvailable={audioAvailable}
        selectedMeasureRange={selectedMeasureRange}
        selectedSequenceRange={selectedSequenceRange}
        measureAudioGenerating={measureAudioGenerating}
        soundfontReady={soundfontReady}
        measureAudioPlaying={measureAudioPlaying}
        playSelectedMeasures={playSelectedMeasures}
        playFromCurrentMeasure={playFromCurrentMeasure}
        stopMeasurePlayback={stopMeasurePlayback}
        shortcutLabel={shortcutLabel}
        playFromCurrentMeasureShortcutLabel={
          playFromCurrentMeasureShortcutLabel
        }
        store={store}
        onSelect={handleSelect}
        onCreate={handleCreate}
        onDuplicate={handleDuplicate}
        onRename={handleRename}
        onDelete={handleDelete}
        onOpenStorageSettings={() => setStorageSettingsOpen(true)}
        saveStatus={saveStatus}
        autosaveDeadline={autosaveDeadline}
        creatingFile={creatingFile}
        deletingFileName={deletingFileName}
        duplicatingFile={duplicatingFile}
        renamingFileName={renamingFileName}
        isLoadingGithub={isLoadingGithub}
        onOpenBin={() => setBinOpen(true)}
        hasDocuments={documents.length > 0}
        rendering={rendering}
        audioGenerating={audioGenerating}
        wavUrl={wavUrl}
        onGenerateAudio={generateFullAudio}
        pdfAvailable={pdfAvailable}
        pdfFontsReady={pdfFontsReady}
        pdfExporting={pdfExporting}
        onExportPdf={exportPdf}
        splitPdfExporting={splitPdfExporting}
        onExportSplitPdf={exportSplitPdf}
        midiAvailable={midiAvailable}
        midiExporting={midiExporting}
        onExportMidi={exportMidi}
        splitMidiExporting={splitMidiExporting}
        onExportSplitMidi={exportSplitMidi}
        splitWavExporting={splitWavExporting}
        onExportSplitWav={exportSplitWav}
        partsCount={parts.length}
        importing={importingFile}
        onImportFile={handleImportFile}
        liveShare={liveShare}
      />
      <AppOverlays
        fileOpError={fileOpError}
        setFileOpError={setFileOpError}
        storageSettingsOpen={storageSettingsOpen}
        setStorageSettingsOpen={setStorageSettingsOpen}
        backend={backend}
        isLoadingGithub={isLoadingGithub}
        preference={preference}
        switchBackend={switchBackend}
        store={store}
        setStore={setStore}
        refreshSaveStatus={refreshSaveStatus}
        selectedMeasureRange={selectedMeasureRange}
        binOpen={binOpen}
        setBinOpen={setBinOpen}
        onRestore={handleRestore}
        restoringFileName={restoringFileName}
      />
      <ExportAudioToast open={audioGenerating} />
      <SectionJumpToolbar {...sectionJumpToolbarProps} />
      <SequenceJumpToolbar {...sequenceJumpToolbarProps} />
      <PartToggles
        parts={parts}
        disabledParts={disabledParts}
        disabledLyrics={disabledLyrics}
        soloedParts={soloedParts}
        onPartToggle={handlePartToggle}
        onLyricsToggle={handleLyricsToggle}
        onSoloToggle={handleSoloToggle}
      />
      <AppWorkspace
        editorCollapsed={editorCollapsed}
        setEditorCollapsed={setEditorCollapsed}
        hideEditor={sharedPreview !== null || liveViewerActive}
        editorRef={editorRef}
        fileId={fileId}
        source={source}
        handleSourceChange={handleSourceChange}
        handleFormatScore={handleFormatScore}
        handleFormatUnzippedText={formatUnzippedText}
        readOnly={readOnly}
        diagnostics={diagnostics}
        diagnosticViewZones={diagnosticViewZones}
        measureSpans={measureSpans}
        setSelectedLineRange={setSelectedLineRange}
        notifySelection={notifySelection}
        notifyUnzippedSelection={notifyUnzippedSelection}
        setEditPartsOpen={setEditPartsOpen}
        setEditMetadataOpen={setEditMetadataOpen}
        forceSave={forceSave}
        measureAudioPlaying={measureAudioPlaying}
        stopMeasurePlayback={stopMeasurePlayback}
        selectedMeasureRange={selectedMeasureRange}
        measureAudioGenerating={measureAudioGenerating}
        soundfontReady={soundfontReady}
        playSelectedMeasures={playSelectedMeasures}
        editPartsOpen={editPartsOpen}
        partDeclarations={partDeclarations}
        parts={parts}
        handlePartDeclarationChange={handlePartDeclarationChange}
        previewInstrument={previewInstrument}
        previewPercussion={previewPercussion}
        stopPreviewInstrument={stopPreviewInstrument}
        previewAudioPlaying={previewAudioPlaying}
        editMetadataOpen={editMetadataOpen}
        parsedMetadata={parsedMetadata}
        handleMetadataFieldChange={handleMetadataFieldChange}
        documents={documents}
        highlightedDocuments={highlightedDocuments}
        rendering={rendering}
        handleMeasureRangeSelect={handleMeasureRangeSelect}
        handleSectionJump={handleSectionJump}
        audioGenerating={audioGenerating}
        wavUrl={wavUrl}
        wavFilename={wavFilename}
        noteTimings={noteTimings}
        measureAudioNoteTimings={measureAudioNoteTimings}
        measureAudioElement={measureAudioElement}
        noPartsSelected={noPartsSelected}
        unzippedView={unzippedView}
        onToggleUnzippedView={handleToggleUnzippedView}
        unzippedText={unzippedText}
        partMeasureRanges={partMeasureRanges}
      />
    </div>
  )
}
