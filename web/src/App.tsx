import { AppHeader } from './components/AppHeader'
import { AppOverlays } from './components/AppOverlays'
import { AppWorkspace } from './components/AppWorkspace'
import { AssetLoadingBanner } from './components/AssetLoadingBanner'
import { ExportAudioToast } from './components/ExportAudioToast'
import { PartToggles } from './components/PartToggles'
import { SectionJumpToolbar } from './components/SectionJumpToolbar'
import { SequenceJumpToolbar } from './components/SequenceJumpToolbar'
import { useAppController } from './hooks/useAppController'
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts'
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
    unzippedView,
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
    notifyUnzippedSelection,
    unzippedText,
    partMeasureRanges,
    playSelectedMeasures,
    playFromCurrentMeasure,
    stopMeasurePlayback,
    highlightedDocuments,
    noteSpans,
    previewInstrument,
    previewPercussion,
    stopPreviewInstrument,
    previewAudioPlaying,
    formatUnzippedText,
    handleSourceChange,
    handleSelect,
    handleFormatScore,
    handleToggleUnzippedView,
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
    handlePlayNoteSelection,
    noPartsSelected,
  } = useAppController()

  useKeyboardShortcuts({
    measureAudioPlaying,
    measureAudioGenerating,
    soundfontReady,
    selectedMeasureRange,
    selectedSequenceRange,
    playSelectedMeasures,
    playFromCurrentMeasure,
    notePlaybackSelectionActive: selectedNoteRangePlaybackInfo !== null,
    playNoteSelection: handlePlayNoteSelection,
    stopMeasurePlayback,
    forceSave,
  })

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
        notePlaybackSelectionActive={selectedNoteRangePlaybackInfo !== null}
        playNoteSelection={handlePlayNoteSelection}
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
        notePlaybackSelectionActive={selectedNoteRangePlaybackInfo !== null}
        playNoteSelection={handlePlayNoteSelection}
        editPartsOpen={editPartsOpen}
        partDeclarations={partDeclarations}
        parts={parts}
        handlePartDeclarationChange={handlePartDeclarationChange}
        handleShiftPartOctave={handleShiftPartOctave}
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
        handleSectionJump={handleSectionJump}
        handleNoteRangeSelect={handleNoteRangeSelect}
        handleEditorSelectionChange={handleEditorSelectionChange}
        selectedNoteCells={selectedNoteCells}
        noteSpans={noteSpans}
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
