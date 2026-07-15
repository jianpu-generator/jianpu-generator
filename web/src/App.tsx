import { useCallback, useMemo, useRef, useState } from 'react'
import { AppHeader } from './components/AppHeader'
import { AppOverlays } from './components/AppOverlays'
import { AppWorkspace } from './components/AppWorkspace'
import { AssetLoadingBanner } from './components/AssetLoadingBanner'
import { SectionJumpToolbar } from './components/SectionJumpToolbar'
import {
  fileContent,
  fileIdForName,
  isReadOnlyFile,
  selectFile,
} from './fileStore'
import { useAssetLoader } from './hooks/useAssetLoader'
import { useFileOperations } from './hooks/useFileOperations'
import { useFontsLoader } from './hooks/useFontsLoader'
import { useJianpuWorker } from './hooks/useJianpuWorker'
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts'
import { usePartTogglePruning } from './hooks/usePartTogglePruning'
import {
  noPartsSelected as computeNoPartsSelected,
  usePartToggles,
} from './hooks/usePartToggles'
import { useSectionNavigation } from './hooks/useSectionNavigation'
import { useSharedPreview } from './hooks/useSharedPreview'
import { useStorageBackend } from './hooks/useStorageBackend'
import type { EditorHandle, PartMode, SoundfontValue } from './types'
import type { MetadataKey } from './utils/metadataSource'
import { parseMetadata, updateMetadataField } from './utils/metadataSource'
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
  } = useStorageBackend()
  const [editPartsOpen, setEditPartsOpen] = useState(false)
  const [editMetadataOpen, setEditMetadataOpen] = useState(false)
  const [storageSettingsOpen, setStorageSettingsOpen] = useState(false)
  const [editorCollapsed, setEditorCollapsed] = useState(false)

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

  const { sharedPreview, handleDismissShared, handleImportShared } =
    useSharedPreview(
      store,
      backend,
      setStore,
      setFileOpError,
      setEditorCollapsed,
    )

  const source = sharedPreview
    ? sharedPreview.content
    : fileContent(store, store.active)
  const readOnly = sharedPreview !== null || isReadOnlyFile(store.active)
  const fileId = fileIdForName(store, store.active)

  const editorRef = useRef<EditorHandle>(null)
  const soundfont = useAssetLoader('/fonts/GeneralUser_GS.sf2')
  const fonts = useFontsLoader()
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
    measureTimes,
    writtenMeasureIndices,
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
    measureAudioTimes,
    measureAudioWrittenIndices,
    measureAudioElement,
    measureSpans,
    sectionRanges,
    notifySelection,
    playSelectedMeasures,
    playFromCurrentMeasure,
    stopMeasurePlayback,
    highlightedDocuments,
    previewInstrument,
    previewPercussion,
    stopPreviewInstrument,
    previewAudioPlaying,
    updatePartDeclaration,
  } = useJianpuWorker(
    source,
    disabledParts,
    disabledLyrics,
    soloedParts,
    store.active,
    soundfont.bytes,
    fonts.fonts,
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
    playSelectedMeasures,
    playFromCurrentMeasure,
    stopMeasurePlayback,
    forceSave,
  })

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

  const handlePartDeclarationChange = useCallback(
    (
      abbreviation: string,
      mode: PartMode,
      followTarget: string | null,
      soundfont: SoundfontValue | null,
      volume: number | null,
      octaveOffset: number | null,
    ) => {
      void updatePartDeclaration(
        abbreviation,
        mode,
        followTarget,
        soundfont,
        volume,
        octaveOffset,
      ).then(handleSourceChange)
    },
    [updatePartDeclaration, handleSourceChange],
  )

  const parsedMetadata = useMemo(() => parseMetadata(source), [source])

  const handleMetadataFieldChange = useCallback(
    (key: MetadataKey, value: string | null) => {
      handleSourceChange(updateMetadataField(source, key, value))
    },
    [source, handleSourceChange],
  )

  const {
    setSelectedLineRange,
    sectionLabels,
    dragStartLabel,
    setDragStartLabel,
    setDragCurrentLabel,
    activeHighlightedLabels,
    handleSectionRangeSelect,
    handleSectionJump,
    handleMeasureRangeSelect,
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
      />
      <AppHeader
        audioAvailable={audioAvailable}
        selectedMeasureRange={selectedMeasureRange}
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
        onRestore={handleRestore}
        restoringFileName={restoringFileName}
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
        selectedMeasureRange={selectedMeasureRange}
        sharedPreview={sharedPreview}
        handleImportShared={handleImportShared}
        handleDismissShared={handleDismissShared}
      />
      <SectionJumpToolbar
        sectionLabels={sectionLabels}
        dragStartLabel={dragStartLabel}
        setDragStartLabel={setDragStartLabel}
        setDragCurrentLabel={setDragCurrentLabel}
        activeHighlightedLabels={activeHighlightedLabels}
        handleSectionJump={handleSectionJump}
        handleSectionRangeSelect={handleSectionRangeSelect}
      />
      <AppWorkspace
        editorCollapsed={editorCollapsed}
        setEditorCollapsed={setEditorCollapsed}
        sharedPreview={sharedPreview}
        editorRef={editorRef}
        fileId={fileId}
        source={source}
        handleSourceChange={handleSourceChange}
        readOnly={readOnly}
        diagnostics={diagnostics}
        diagnosticViewZones={diagnosticViewZones}
        measureSpans={measureSpans}
        setSelectedLineRange={setSelectedLineRange}
        notifySelection={notifySelection}
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
        measureTimes={measureTimes}
        writtenMeasureIndices={writtenMeasureIndices}
        measureAudioTimes={measureAudioTimes}
        measureAudioWrittenIndices={measureAudioWrittenIndices}
        measureAudioElement={measureAudioElement}
        noPartsSelected={noPartsSelected}
        disabledParts={disabledParts}
        disabledLyrics={disabledLyrics}
        soloedParts={soloedParts}
        handlePartToggle={handlePartToggle}
        handleLyricsToggle={handleLyricsToggle}
        handleSoloToggle={handleSoloToggle}
      />
    </div>
  )
}
