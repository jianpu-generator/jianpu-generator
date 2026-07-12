import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { AssetLoadingBanner } from './components/AssetLoadingBanner'
import { BinMenu } from './components/BinMenu'
import { EditMetadataModal } from './components/EditMetadataModal'
import { Editor } from './components/Editor'
import { EditPartsModal } from './components/EditPartsModal'
import { ErrorModal } from './components/ErrorModal'
import { ExportControls } from './components/ExportControls'
import { FileSwitcher } from './components/FileSwitcher'
import { PartToggles } from './components/PartToggles'
import { PlayFromCurrentMeasureButton } from './components/PlayFromCurrentMeasureButton'
import { PlayMeasureButton } from './components/PlayMeasureButton'
import { Preview } from './components/Preview'
import { SharedPreviewBanner } from './components/SharedPreviewBanner'
import { StorageSettingsModal } from './components/StorageSettingsModal'
import {
  fileContent,
  fileIdForName,
  isReadOnlyFile,
  mergeBackendResult,
  selectFile,
  sortedBinNames,
} from './fileStore'
import { useAssetLoader } from './hooks/useAssetLoader'
import { useFileOperations } from './hooks/useFileOperations'
import { useFontsLoader } from './hooks/useFontsLoader'
import { useJianpuWorker } from './hooks/useJianpuWorker'
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts'
import { usePartToggles } from './hooks/usePartToggles'
import { useSectionNavigation } from './hooks/useSectionNavigation'
import { useStorageBackend } from './hooks/useStorageBackend'
import {
  clearShareHash,
  parseShareFromHash,
  type SharePayload,
} from './shareUrl'
import type { EditorHandle, PartMode, SoundfontValue } from './types'
import type { MetadataKey } from './utils/metadataSource'
import { parseMetadata, updateMetadataField } from './utils/metadataSource'
import './App.css'
import './file-switcher.css'
import './preview.css'

const shortcutLabel = navigator.platform.startsWith('Mac') ? '⌘↵' : 'Ctrl+↵'
const playFromCurrentMeasureShortcutLabel = navigator.platform.startsWith('Mac')
  ? '⇧⌘↵'
  : 'Ctrl+Shift+↵'

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
  const [sharedPreview, setSharedPreview] = useState<SharePayload | null>(null)
  useEffect(() => {
    let cancelled = false
    void parseShareFromHash().then((parsed) => {
      if (!cancelled) {
        setSharedPreview(parsed)
        if (parsed) setEditorCollapsed(true)
      }
    })
    return () => {
      cancelled = true
    }
  }, [])
  const source = sharedPreview
    ? sharedPreview.content
    : fileContent(store, store.active)
  const readOnly = sharedPreview !== null || isReadOnlyFile(store.active)
  const fileId = fileIdForName(store, store.active)

  const [editPartsOpen, setEditPartsOpen] = useState(false)
  const [editMetadataOpen, setEditMetadataOpen] = useState(false)
  const [storageSettingsOpen, setStorageSettingsOpen] = useState(false)
  const [editorCollapsed, setEditorCollapsed] = useState(false)
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
    measureAudioElement,
    measureSpans,
    sectionRanges,
    notifySelection,
    playSelectedMeasures,
    playFromCurrentMeasure,
    stopMeasurePlayback,
    highlightedDocuments,
    previewInstrument,
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

  useEffect(() => {
    if (parts.length === 0) return

    const abbreviations = new Set(parts.map((part) => part.abbreviation))
    setDisabledParts((prev) => {
      const next = new Set(
        [...prev].filter((abbreviation) => abbreviations.has(abbreviation)),
      )
      return next.size === prev.size ? prev : next
    })
  }, [parts, setDisabledParts])

  useEffect(() => {
    if (parts.length === 0) return

    const lyricAbbreviations = new Set(
      parts.filter((part) => part.has_lyrics).map((part) => part.abbreviation),
    )
    setDisabledLyrics((prev) => {
      const next = new Set(
        [...prev].filter((abbreviation) =>
          lyricAbbreviations.has(abbreviation),
        ),
      )
      return next.size === prev.size ? prev : next
    })
  }, [parts, setDisabledLyrics])

  useEffect(() => {
    if (parts.length === 0) return
    const abbreviations = new Set(parts.map((part) => part.abbreviation))
    setSoloedParts((prev) => {
      const next = new Set([...prev].filter((abbr) => abbreviations.has(abbr)))
      return next.size === prev.size ? prev : next
    })
  }, [parts, setSoloedParts])

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

  const handleDismissShared = useCallback(() => {
    clearShareHash()
    setSharedPreview(null)
  }, [])

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

  const handleImportShared = useCallback(async () => {
    if (!sharedPreview) return
    const base = store
    try {
      const next = await backend.importFile(
        base,
        sharedPreview.filename,
        sharedPreview.content,
      )
      setStore((prev) => mergeBackendResult(prev, base, next))
      clearShareHash()
      setSharedPreview(null)
    } catch (error) {
      setFileOpError({
        title: 'Could not import shared score',
        message: error instanceof Error ? error.message : String(error),
        stack: error instanceof Error ? error.stack : undefined,
      })
    }
  }, [sharedPreview, store, backend, setStore, setFileOpError])

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

  const noPartsSelected =
    parts.length > 0 &&
    soloedParts.size === 0 &&
    parts.every((part) => disabledParts.has(part.abbreviation))

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
      <header className="app-header">
        <h1>簡譜</h1>
        <span className="app-subtitle">live preview</span>
        {audioAvailable && (
          <PlayMeasureButton
            disabled={
              selectedMeasureRange === null ||
              measureAudioGenerating ||
              !soundfontReady
            }
            loading={measureAudioGenerating}
            playing={measureAudioPlaying}
            measureRange={selectedMeasureRange}
            onClick={playSelectedMeasures}
            onPause={stopMeasurePlayback}
            shortcutLabel={shortcutLabel}
          />
        )}
        {audioAvailable && (
          <PlayFromCurrentMeasureButton
            disabled={
              selectedMeasureRange === null ||
              measureAudioGenerating ||
              !soundfontReady
            }
            loading={measureAudioGenerating}
            playing={measureAudioPlaying}
            currentMeasure={selectedMeasureRange?.start ?? null}
            onClick={playFromCurrentMeasure}
            onPause={stopMeasurePlayback}
            shortcutLabel={playFromCurrentMeasureShortcutLabel}
          />
        )}
        <div className="app-header-actions">
          <FileSwitcher
            store={store}
            onSelect={handleSelect}
            onCreate={handleCreate}
            onDuplicate={handleDuplicate}
            onRename={handleRename}
            onDelete={handleDelete}
            onOpenStorageSettings={() => setStorageSettingsOpen(true)}
            saveStatus={saveStatus}
            autosaveDeadline={autosaveDeadline}
            creating={creatingFile}
            deletingName={deletingFileName}
            duplicating={duplicatingFile}
            renamingName={renamingFileName}
            isLoadingGithub={isLoadingGithub}
          />
          <BinMenu
            binNames={sortedBinNames(store)}
            onRestore={handleRestore}
            restoringName={restoringFileName}
          />
          <ExportControls
            hasDocuments={documents.length > 0}
            rendering={rendering}
            audioGenerating={audioGenerating}
            wavUrl={wavUrl}
            soundfontReady={soundfontReady}
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
            audioAvailable={audioAvailable}
            splitWavExporting={splitWavExporting}
            onExportSplitWav={exportSplitWav}
            partsCount={parts.length}
            isLoadingGithub={isLoadingGithub}
          />
        </div>
      </header>
      <ErrorModal
        open={fileOpError !== null}
        onOpenChange={(open) => {
          if (!open) setFileOpError(null)
        }}
        title={fileOpError?.title ?? ''}
        message={fileOpError?.message ?? ''}
        stack={fileOpError?.stack}
      />
      <StorageSettingsModal
        open={storageSettingsOpen}
        onOpenChange={setStorageSettingsOpen}
        backend={backend}
        isLoadingGithub={isLoadingGithub}
        preference={preference}
        switchBackend={switchBackend}
        store={store}
        setStore={setStore}
      />
      <span
        data-testid="selected-measure-range"
        aria-hidden="true"
        style={{ display: 'none' }}
      >
        {selectedMeasureRange
          ? `${selectedMeasureRange.start}-${selectedMeasureRange.end}`
          : ''}
      </span>
      {sharedPreview ? (
        <SharedPreviewBanner
          filename={sharedPreview.filename}
          onImport={handleImportShared}
          onDiscard={handleDismissShared}
        />
      ) : null}
      {sectionLabels.length > 0 ? (
        <div className="workspace-toolbar">
          <div
            role="toolbar"
            className="workspace-toolbar-sections"
            style={{
              userSelect: dragStartLabel !== null ? 'none' : undefined,
            }}
            onMouseDown={(e) => e.preventDefault()}
            onMouseUp={() => {
              setDragStartLabel(null)
              setDragCurrentLabel(null)
            }}
            onMouseLeave={() => {
              setDragStartLabel(null)
              setDragCurrentLabel(null)
            }}
          >
            {sectionLabels.map((label) => (
              <button
                key={label}
                type="button"
                className={[
                  'section-jump-btn',
                  activeHighlightedLabels.has(label)
                    ? 'section-jump-btn--dragging'
                    : '',
                ].join(' ')}
                style={{
                  cursor: dragStartLabel !== null ? 'ew-resize' : undefined,
                }}
                onMouseDown={() => {
                  setDragStartLabel(label)
                  setDragCurrentLabel(label)
                  handleSectionJump(label)
                }}
                onMouseEnter={() => {
                  if (dragStartLabel !== null) {
                    setDragCurrentLabel(label)
                    handleSectionRangeSelect(dragStartLabel, label)
                  }
                }}
              >
                {label}
              </button>
            ))}
          </div>
        </div>
      ) : null}
      <main className="workspace">
        <section
          className={[
            'pane',
            'pane--editor',
            editorCollapsed ? 'pane--editor-collapsed' : '',
          ]
            .filter(Boolean)
            .join(' ')}
        >
          <div className="editor-layout">
            <div className="editor-main">
              {sharedPreview ? null : (
                <Editor
                  ref={editorRef}
                  path={fileId}
                  value={source}
                  onChange={handleSourceChange}
                  readOnly={readOnly}
                  diagnostics={diagnostics}
                  diagnosticViewZones={diagnosticViewZones}
                  measureSpans={measureSpans}
                  onSelectionChange={(firstLine, lastLine) => {
                    setSelectedLineRange(null)
                    notifySelection(firstLine, lastLine)
                  }}
                  onEditPartsClick={() => setEditPartsOpen(true)}
                  onEditMetadataClick={() => setEditMetadataOpen(true)}
                  onForceSave={forceSave}
                  onPlayMeasure={
                    measureAudioPlaying
                      ? stopMeasurePlayback
                      : selectedMeasureRange !== null &&
                          !measureAudioGenerating &&
                          soundfontReady
                        ? playSelectedMeasures
                        : undefined
                  }
                />
              )}
              <EditPartsModal
                open={editPartsOpen}
                onOpenChange={setEditPartsOpen}
                partDeclarations={partDeclarations}
                allParts={parts}
                onPartDeclarationChange={handlePartDeclarationChange}
                previewInstrument={previewInstrument}
                stopPreviewInstrument={stopPreviewInstrument}
                previewAudioPlaying={previewAudioPlaying}
              />
              <EditMetadataModal
                open={editMetadataOpen}
                onOpenChange={setEditMetadataOpen}
                metadata={parsedMetadata}
                onFieldChange={handleMetadataFieldChange}
              />
            </div>
          </div>
        </section>
        <div className="pane-divider">
          {sharedPreview ? null : (
            <button
              type="button"
              className="pane-divider-toggle"
              onClick={() => setEditorCollapsed((collapsed) => !collapsed)}
              title={editorCollapsed ? 'Show editor' : 'Hide editor'}
              aria-label={editorCollapsed ? 'Show editor' : 'Hide editor'}
            >
              <span
                className="pane-divider-toggle-icon"
                style={{
                  transform: editorCollapsed ? 'rotate(180deg)' : 'none',
                }}
                aria-hidden="true"
              >
                ‹
              </span>
            </button>
          )}
        </div>
        <section className="pane pane--preview">
          <Preview
            documents={documents}
            highlightedDocuments={highlightedDocuments}
            rendering={rendering}
            onMeasureRangeSelect={handleMeasureRangeSelect}
            onSectionLabelClick={handleSectionJump}
            audioGenerating={audioGenerating}
            wavUrl={wavUrl}
            wavFilename={wavFilename}
            measureTimes={measureTimes}
            measureAudioTimes={measureAudioTimes}
            measureAudioElement={measureAudioElement}
            selectedMeasureRange={selectedMeasureRange}
            emptyMessage={
              noPartsSelected ? 'No parts selected.' : 'No preview yet.'
            }
            toolbar={
              <PartToggles
                parts={parts}
                disabledParts={disabledParts}
                disabledLyrics={disabledLyrics}
                soloedParts={soloedParts}
                onPartToggle={handlePartToggle}
                onLyricsToggle={handleLyricsToggle}
                onSoloToggle={handleSoloToggle}
              />
            }
          />
        </section>
      </main>
    </div>
  )
}
