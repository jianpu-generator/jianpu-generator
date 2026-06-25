import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { AssetLoadingBanner } from './components/AssetLoadingBanner'
import { Editor } from './components/Editor'
import { EditPartsModal } from './components/EditPartsModal'
import { FileTabBar } from './components/FileList'
import { PartToggles } from './components/PartToggles'
import { PlayMeasureButton } from './components/PlayMeasureButton'
import { Preview } from './components/Preview'
import {
  createFile,
  deleteFile,
  duplicateFile,
  fileContent,
  fileIdForName,
  isReadOnlyFile,
  renameFile,
  restoreFile,
  selectFile,
  updateActiveContent,
} from './fileStore'
import { useAssetLoader } from './hooks/useAssetLoader'
import { useFileStore } from './hooks/useFileStore'
import { useFontsLoader } from './hooks/useFontsLoader'
import { useJianpuWorker } from './hooks/useJianpuWorker'
import {
  readPartTogglesForFile,
  writePartTogglesForFile,
} from './partToggleCache'
import type { EditorHandle } from './types'
import { byteOffsetToStringIndex } from './utils/byteSpan'
import type { PartMode, SoundfontValue } from './utils/partSource'
import {
  parsePartDeclarations,
  updatePartDeclaration,
} from './utils/partSource'
import './App.css'
import './file-tab-bar.css'
import './preview.css'

const shortcutLabel = navigator.platform.startsWith('Mac') ? '⌘↵' : 'Ctrl+↵'

interface SectionLabel {
  label: string
  byteOffset: number
}

export default function App() {
  const [store, setStore] = useFileStore()
  const source = fileContent(store, store.active)
  const readOnly = isReadOnlyFile(store.active)
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
  const editorRef = useRef<EditorHandle>(null)
  const skipToggleSaveRef = useRef(false)
  const soundfont = useAssetLoader('/fonts/GeneralUser_GS.sf2')
  const fonts = useFontsLoader()
  const soundfontReady = soundfont.status === 'ready'
  const pdfFontsReady = fonts.status === 'ready'
  const {
    parts,
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
    notifySelection,
    playSelectedMeasures,
    stopMeasurePlayback,
    highlightedDocuments,
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
    : selectedMeasureRange !== null && !measureAudioGenerating
      ? playSelectedMeasures
      : undefined

  useEffect(() => {
    const isMac = navigator.platform.startsWith('Mac')
    const onKeyDown = (event: KeyboardEvent) => {
      const modifier = isMac ? event.metaKey : event.ctrlKey
      if (modifier && event.key === 'Enter') {
        event.preventDefault()
        playMeasureRef.current?.()
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
      setStore((prev) => updateActiveContent(prev, value))
    },
    [setStore],
  )

  const handleSelect = useCallback(
    (name: string) => {
      setStore((prev) => selectFile(prev, name))
    },
    [setStore],
  )

  const handleCreate = useCallback(() => {
    setStore((prev) => createFile(prev))
  }, [setStore])

  const handleDuplicate = useCallback(() => {
    setStore((prev) => duplicateFile(prev))
  }, [setStore])

  const handleRename = useCallback(
    (from: string, to: string) => {
      setStore((prev) => renameFile(prev, from, to))
    },
    [setStore],
  )

  const handleDelete = useCallback(
    (name: string) => {
      setStore((prev) => deleteFile(prev, name))
    },
    [setStore],
  )

  const handleRestore = useCallback(
    (name: string) => {
      setStore((prev) => restoreFile(prev, name))
    },
    [setStore],
  )

  const partDeclarations = useMemo(
    () => parsePartDeclarations(source, parts),
    [source, parts],
  )

  const handlePartDeclarationChange = useCallback(
    (
      abbreviation: string,
      mode: PartMode,
      followTarget: string | null,
      soundfont: SoundfontValue | null,
    ) => {
      const newSource = updatePartDeclaration(
        source,
        abbreviation,
        mode,
        followTarget,
        soundfont,
      )
      handleSourceChange(newSource)
    },
    [source, handleSourceChange],
  )

  const sectionLabels = useMemo<SectionLabel[]>(() => {
    const seen = new Set<string>()
    const result: SectionLabel[] = []
    for (const span of measureSpans) {
      if (span.section_label != null && !seen.has(span.section_label)) {
        seen.add(span.section_label)
        result.push({
          label: span.section_label,
          byteOffset: span.view_zone_start,
        })
      }
    }
    return result
  }, [measureSpans])

  const handleSectionJump = useCallback(
    (byteOffset: number) => {
      const charOffset = byteOffsetToStringIndex(source, byteOffset)
      editorRef.current?.jumpToOffset(charOffset)
    },
    [source],
  )

  const handleMeasureClick = useCallback(
    (measureIndex: number) => {
      const span = measureSpans[measureIndex]
      if (!span) return
      const charOffset = byteOffsetToStringIndex(source, span.start)
      editorRef.current?.jumpToOffset(charOffset)
    },
    [measureSpans, source],
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
      />
      <main className="workspace">
        <section className="pane pane--editor">
          <div className="editor-layout">
            <div className="editor-main">
              <Editor
                ref={editorRef}
                value={source}
                onChange={handleSourceChange}
                readOnly={readOnly}
                diagnostics={diagnostics}
                diagnosticViewZones={diagnosticViewZones}
                measureSpans={measureSpans}
                onSelectionChange={notifySelection}
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
                  audioAvailable ||
                  sectionLabels.length > 0 ||
                  partDeclarations.length > 0 ? (
                    <div
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: '0.5rem',
                      }}
                    >
                      {partDeclarations.length > 0 && (
                        <button
                          type="button"
                          className="section-jump-btn"
                          data-testid="edit-parts-btn"
                          onClick={() => setEditPartsOpen(true)}
                        >
                          Edit Parts
                        </button>
                      )}
                      {audioAvailable && (
                        <PlayMeasureButton
                          disabled={
                            selectedMeasureRange === null ||
                            measureAudioGenerating
                          }
                          loading={measureAudioGenerating}
                          playing={measureAudioPlaying}
                          measureRange={selectedMeasureRange}
                          onClick={playSelectedMeasures}
                          onPause={stopMeasurePlayback}
                          shortcutLabel={shortcutLabel}
                        />
                      )}
                      {sectionLabels.length > 0 && (
                        <div
                          style={{
                            display: 'flex',
                            alignItems: 'center',
                            gap: '0.25rem',
                            overflowX: 'auto',
                            flexShrink: 1,
                          }}
                        >
                          {sectionLabels.map(({ label, byteOffset }) => (
                            <button
                              key={label}
                              type="button"
                              className="section-jump-btn"
                              onClick={() => handleSectionJump(byteOffset)}
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
              <EditPartsModal
                open={editPartsOpen}
                onOpenChange={setEditPartsOpen}
                partDeclarations={partDeclarations}
                allParts={parts}
                onPartDeclarationChange={handlePartDeclarationChange}
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
            onMeasureClick={handleMeasureClick}
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
