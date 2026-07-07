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
  type FileStoreState,
  isReadOnlyFile,
  mergeBackendResult,
  selectFile,
} from './fileStore'
import { useAssetLoader } from './hooks/useAssetLoader'
import { useFontsLoader } from './hooks/useFontsLoader'
import { useJianpuWorker } from './hooks/useJianpuWorker'
import { useStorageBackend } from './hooks/useStorageBackend'
import {
  readPartTogglesForFile,
  writePartTogglesForFile,
} from './partToggleCache'
import { clearShareHash, parseShareFromHash, type SharePayload } from './shareUrl'
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
  } = useStorageBackend()
  const [sharedPreview, setSharedPreview] = useState<SharePayload | null>(() =>
    parseShareFromHash(),
  )
  const source = sharedPreview
    ? sharedPreview.content
    : fileContent(store, store.active)
  const readOnly = sharedPreview !== null || isReadOnlyFile(store.active)
  const fileId = fileIdForName(store, store.active)

  const [disabledParts, setDisabledParts] = useState<Set<string>>(() => {
    const cached = readPartTogglesForFile(fileId)
    return new Set(cached?.disabledParts ?? [])
  })
  const [disabledLyrics, setDisabledLyrics] = useState<Set<string>>(() => {
    const cached = readPartTogglesForFile(fileId)
    return new Set(cached?.disabledLyrics ?? [])
  })
  const [soloedParts, setSoloedParts] = useState<Set<string>>(() => {
    const cached = readPartTogglesForFile(fileId)
    return new Set(cached?.soloedParts ?? [])
  })
  const [editPartsOpen, setEditPartsOpen] = useState(false)
  const [editMetadataOpen, setEditMetadataOpen] = useState(false)
  const [storageSettingsOpen, setStorageSettingsOpen] = useState(false)
  const [creatingFile, setCreatingFile] = useState(false)
  const [deletingFileName, setDeletingFileName] = useState<string | null>(null)
  const [duplicatingFile, setDuplicatingFile] = useState(false)
  const [renamingFileName, setRenamingFileName] = useState<string | null>(null)
  const [restoringFileName, setRestoringFileName] = useState<string | null>(
    null,
  )
  const [fileOpError, setFileOpError] = useState<{
    title: string
    message: string
    stack?: string
  } | null>(null)
  const [dragStartLabel, setDragStartLabel] = useState<string | null>(null)
  const [dragCurrentLabel, setDragCurrentLabel] = useState<string | null>(null)
  const [selectedLineRange, setSelectedLineRange] = useState<{
    firstLine: number
    lastLine: number
  } | null>(null)
  const editorRef = useRef<EditorHandle>(null)
  const skipToggleSaveRef = useRef(false)
  const soundfont = useAssetLoader('/fonts/GeneralUser_GS.sf2')
  const fonts = useFontsLoader()
  const soundfontReady = soundfont.status === 'ready'
  const pdfFontsReady = fonts.status === 'ready'
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
    skipToggleSaveRef.current = true
    const cached = readPartTogglesForFile(fileId)
    setDisabledParts(new Set(cached?.disabledParts ?? []))
    setDisabledLyrics(new Set(cached?.disabledLyrics ?? []))
    setSoloedParts(new Set(cached?.soloedParts ?? []))
  }, [fileId])

  useEffect(() => {
    if (skipToggleSaveRef.current) {
      skipToggleSaveRef.current = false
      return
    }
    writePartTogglesForFile(fileId, {
      disabledParts: [...disabledParts],
      disabledLyrics: [...disabledLyrics],
      soloedParts: [...soloedParts],
    })
  }, [fileId, disabledParts, disabledLyrics, soloedParts])

  useEffect(() => {
    if (parts.length === 0) return

    const abbreviations = new Set(parts.map((part) => part.abbreviation))
    setDisabledParts((prev) => {
      const next = new Set(
        [...prev].filter((abbreviation) => abbreviations.has(abbreviation)),
      )
      return next.size === prev.size ? prev : next
    })
  }, [parts])

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
  }, [parts])

  useEffect(() => {
    if (parts.length === 0) return
    const abbreviations = new Set(parts.map((part) => part.abbreviation))
    setSoloedParts((prev) => {
      const next = new Set([...prev].filter((abbr) => abbreviations.has(abbr)))
      return next.size === prev.size ? prev : next
    })
  }, [parts])

  const playMeasureRef = useRef<(() => void) | undefined>(undefined)
  playMeasureRef.current = measureAudioPlaying
    ? stopMeasurePlayback
    : selectedMeasureRange !== null &&
        !measureAudioGenerating &&
        soundfontReady
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

  const handlePartToggle = useCallback(
    (abbreviation: string, enabled: boolean) => {
      setDisabledParts((prev) => {
        const next = new Set(prev)
        if (enabled) {
          next.delete(abbreviation)
        } else {
          next.add(abbreviation)
        }
        return next
      })
    },
    [],
  )

  const handleLyricsToggle = useCallback(
    (abbreviation: string, enabled: boolean) => {
      setDisabledLyrics((prev) => {
        const next = new Set(prev)
        if (enabled) {
          next.delete(abbreviation)
        } else {
          next.add(abbreviation)
        }
        return next
      })
    },
    [],
  )

  const handleSoloToggle = useCallback(
    (abbreviation: string, soloed: boolean) => {
      setSoloedParts((prev) => {
        const next = new Set(prev)
        if (soloed) {
          next.add(abbreviation)
        } else {
          next.delete(abbreviation)
        }
        return next
      })
    },
    [],
  )

  const handleSourceChange = useCallback(
    (value: string) => {
      setStore((prev) => backend.updateActiveContent(prev, value))
    },
    [setStore, backend],
  )

  const handleSelect = useCallback(
    (name: string) => {
      setStore((prev) => selectFile(prev, name))
    },
    [setStore],
  )

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
  }, [sharedPreview, store, backend, setStore])

  const handleDismissShared = useCallback(() => {
    clearShareHash()
    setSharedPreview(null)
  }, [])

  const runFileOp = useCallback(
    async (
      errorTitle: string,
      setPending: (pending: boolean) => void,
      op: (base: FileStoreState) => Promise<FileStoreState>,
    ) => {
      const base = store
      setPending(true)
      try {
        const next = await op(base)
        setStore((prev) => mergeBackendResult(prev, base, next))
      } catch (error) {
        setFileOpError({
          title: errorTitle,
          message: error instanceof Error ? error.message : String(error),
          stack: error instanceof Error ? error.stack : undefined,
        })
      } finally {
        setPending(false)
      }
    },
    [setStore, store],
  )

  const handleCreate = useCallback(
    () =>
      runFileOp('Could not create file', setCreatingFile, (base) =>
        backend.createFile(base),
      ),
    [runFileOp, backend],
  )

  const handleDuplicate = useCallback(
    () =>
      runFileOp('Could not duplicate file', setDuplicatingFile, (base) =>
        backend.duplicateFile(base),
      ),
    [runFileOp, backend],
  )

  const handleRename = useCallback(
    (from: string, to: string) =>
      runFileOp(
        'Could not rename file',
        (pending) => setRenamingFileName(pending ? from : null),
        (base) => backend.renameFile(base, from, to),
      ),
    [runFileOp, backend],
  )

  const handleDelete = useCallback(
    (name: string) =>
      runFileOp(
        'Could not delete file',
        (pending) => setDeletingFileName(pending ? name : null),
        (base) => backend.deleteFile(base, name),
      ),
    [runFileOp, backend],
  )

  const handleRestore = useCallback(
    (name: string) =>
      runFileOp(
        'Could not restore file',
        (pending) => setRestoringFileName(pending ? name : null),
        (base) => backend.restoreFile(base, name),
      ),
    [runFileOp, backend],
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

  const sectionLabels = useMemo(
    () =>
      sectionRanges
        .filter((r) => r.labels.length === 1)
        .flatMap((r) => r.labels),
    [sectionRanges],
  )

  const dragHighlightedLabels = useMemo<Set<string>>(() => {
    if (dragStartLabel === null || dragCurrentLabel === null) return new Set()
    const a = sectionLabels.indexOf(dragStartLabel)
    const b = sectionLabels.indexOf(dragCurrentLabel)
    if (a === -1 || b === -1) return new Set()
    return new Set(sectionLabels.slice(Math.min(a, b), Math.max(a, b) + 1))
  }, [dragStartLabel, dragCurrentLabel, sectionLabels])

  const activeHighlightedLabels = useMemo(() => {
    if (dragStartLabel !== null) return dragHighlightedLabels
    if (!selectedLineRange) return new Set<string>()
    const match = sectionRanges.find(
      (r) =>
        r.first_line === selectedLineRange.firstLine &&
        r.last_line === selectedLineRange.lastLine,
    )
    return new Set(match?.labels ?? [])
  }, [dragStartLabel, dragHighlightedLabels, selectedLineRange, sectionRanges])

  const selectSectionRange = useCallback(
    (firstLine: number, lastLine: number) => {
      editorRef.current?.setSelectionByLines(firstLine, lastLine)
      editorRef.current?.focus()
      setSelectedLineRange({ firstLine, lastLine })
      notifySelection(firstLine, lastLine)
    },
    [notifySelection],
  )

  const handleSectionRangeSelect = useCallback(
    (labelA: string, labelB: string) => {
      const range =
        sectionRanges.find(
          (r) => r.labels[0] === labelA && r.labels.at(-1) === labelB,
        ) ??
        sectionRanges.find(
          (r) => r.labels[0] === labelB && r.labels.at(-1) === labelA,
        )
      if (!range) return
      selectSectionRange(range.first_line, range.last_line)
    },
    [sectionRanges, selectSectionRange],
  )

  const handleSectionJump = useCallback(
    (label: string) => handleSectionRangeSelect(label, label),
    [handleSectionRangeSelect],
  )

  useEffect(() => {
    const clearDrag = () => {
      setDragStartLabel(null)
      setDragCurrentLabel(null)
    }
    window.addEventListener('mouseup', clearDrag)
    return () => window.removeEventListener('mouseup', clearDrag)
  }, [])

  const handleMeasureRangeSelect = useCallback(
    (start: number, end: number) => {
      const s = measureSpans[start]
      const e = measureSpans[end]
      if (!s || !e) return
      editorRef.current?.setSelectionByLines(s.start_line, e.end_line)
    },
    [measureSpans],
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
