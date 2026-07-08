import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { AssetLoadingBanner } from './components/AssetLoadingBanner'
import { EditMetadataModal } from './components/EditMetadataModal'
import { Editor } from './components/Editor'
import { EditPartsModal } from './components/EditPartsModal'
import { ErrorModal } from './components/ErrorModal'
import { FileTabBar } from './components/FileList'
import { PartToggles } from './components/PartToggles'
import { PlayMeasureButton } from './components/PlayMeasureButton'
import { Preview } from './components/Preview'
import { StorageSettingsModal } from './components/StorageSettingsModal'
import {
  fileContent,
  fileIdForName,
  isReadOnlyFile,
  mergeBackendResult,
  selectFile,
} from './fileStore'
import { useAssetLoader } from './hooks/useAssetLoader'
import { useFileOperations } from './hooks/useFileOperations'
import { useFontsLoader } from './hooks/useFontsLoader'
import { useJianpuWorker } from './hooks/useJianpuWorker'
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
import './file-tab-bar.css'
import './preview.css'

const shortcutLabel = navigator.platform.startsWith('Mac') ? '⌘↵' : 'Ctrl+↵'

export default function App() {
  const {
    store,
    setStore,
    backend,
    saveStatus,
    preference,
    switchBackend,
    forceSave,
    flushPendingSave,
    refreshSaveStatus,
  } = useStorageBackend()
  const [sharedPreview, setSharedPreview] = useState<SharePayload | null>(null)
  useEffect(() => {
    let cancelled = false
    void parseShareFromHash().then((parsed) => {
      if (!cancelled) setSharedPreview(parsed)
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
    partsLoading,
    documents,
    wavUrl,
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
    generateFullAudio,
    selectedMeasureRange,
    measureAudioGenerating,
    measureAudioPlaying,
    measureSpans,
    sectionRanges,
    notifySelection,
    playSelectedMeasures,
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

  const playMeasureRef = useRef<(() => void) | undefined>(undefined)
  playMeasureRef.current = measureAudioPlaying
    ? stopMeasurePlayback
    : selectedMeasureRange !== null && !measureAudioGenerating && soundfontReady
      ? playSelectedMeasures
      : undefined

  const forceSaveRef = useRef(forceSave)
  forceSaveRef.current = forceSave

  useEffect(() => {
    const isMac = navigator.platform.startsWith('Mac')
    const onKeyDown = (event: KeyboardEvent) => {
      const modifier = isMac ? event.metaKey : event.ctrlKey
      if (modifier && event.key === 'Enter') {
        event.preventDefault()
        playMeasureRef.current?.()
      } else if (modifier && event.key.toLowerCase() === 's') {
        event.preventDefault()
        forceSaveRef.current()
      }
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [])

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
      </header>
      <FileTabBar
        store={store}
        onSelect={handleSelect}
        onCreate={handleCreate}
        onDuplicate={handleDuplicate}
        onRename={handleRename}
        onDelete={handleDelete}
        onRestore={handleRestore}
        onOpenStorageSettings={() => setStorageSettingsOpen(true)}
        saveStatus={saveStatus}
        creating={creatingFile}
        deletingName={deletingFileName}
        duplicating={duplicatingFile}
        renamingName={renamingFileName}
        restoringName={restoringFileName}
      />
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
        preference={preference}
        switchBackend={switchBackend}
        store={store}
        setStore={setStore}
        refreshSaveStatus={refreshSaveStatus}
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
      <main className="workspace">
        <section className="pane pane--editor">
          <div className="editor-layout">
            <div className="editor-main">
              {sharedPreview ? (
                <div className="shared-preview-banner">
                  <p>
                    Viewing a shared score:{' '}
                    <strong>{sharedPreview.filename}</strong>
                  </p>
                  <div className="shared-preview-actions">
                    <button
                      type="button"
                      className="shared-preview-import-btn"
                      onClick={handleImportShared}
                    >
                      Import to my scores
                    </button>
                    <button
                      type="button"
                      className="shared-preview-discard-btn"
                      onClick={handleDismissShared}
                    >
                      Discard
                    </button>
                  </div>
                </div>
              ) : (
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
                  toolbar={
                    audioAvailable || sectionLabels.length > 0 ? (
                      <div
                        style={{
                          display: 'flex',
                          alignItems: 'center',
                          gap: '0.5rem',
                        }}
                      >
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
                          <span
                            data-testid="measure-status"
                            style={{
                              fontSize: '0.75rem',
                              color: '#888',
                              fontFamily: 'monospace',
                            }}
                          >
                            {selectedMeasureRange !== null
                              ? selectedMeasureRange.start ===
                                selectedMeasureRange.end
                                ? `measure ${selectedMeasureRange.start + 1}`
                                : `measures ${selectedMeasureRange.start + 1}–${selectedMeasureRange.end + 1}`
                              : 'measure null'}
                          </span>
                        )}
                        {sectionLabels.length > 0 && (
                          <div
                            role="toolbar"
                            style={{
                              display: 'flex',
                              alignItems: 'center',
                              gap: '0.25rem',
                              overflowX: 'auto',
                              flexShrink: 1,
                              userSelect:
                                dragStartLabel !== null ? 'none' : undefined,
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
                                  cursor:
                                    dragStartLabel !== null
                                      ? 'ew-resize'
                                      : undefined,
                                }}
                                onMouseDown={() => {
                                  setDragStartLabel(label)
                                  setDragCurrentLabel(label)
                                  handleSectionJump(label)
                                }}
                                onMouseEnter={() => {
                                  if (dragStartLabel !== null) {
                                    setDragCurrentLabel(label)
                                    handleSectionRangeSelect(
                                      dragStartLabel,
                                      label,
                                    )
                                  }
                                }}
                              >
                                {label}
                              </button>
                            ))}
                          </div>
                        )}
                      </div>
                    ) : null
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
        <div className="pane-divider" aria-hidden="true" />
        <section className="pane pane--preview">
          <Preview
            documents={documents}
            highlightedDocuments={highlightedDocuments}
            rendering={rendering}
            onMeasureRangeSelect={handleMeasureRangeSelect}
            onSectionLabelClick={handleSectionJump}
            audioGenerating={audioGenerating}
            wavUrl={wavUrl}
            audioAvailable={audioAvailable}
            soundfontReady={soundfontReady}
            onGenerateAudio={generateFullAudio}
            pdfAvailable={pdfAvailable}
            pdfFontsReady={pdfFontsReady}
            pdfExporting={pdfExporting}
            onExportPdf={exportPdf}
            splitPdfExporting={splitPdfExporting}
            onExportSplitPdf={exportSplitPdf}
            partsCount={parts.length}
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
                loading={partsLoading}
              />
            }
          />
        </section>
      </main>
    </div>
  )
}
