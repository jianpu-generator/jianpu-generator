import { useCallback, useEffect, useRef, useState } from 'react'
import { Editor } from './components/Editor'
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
import { useFileStore } from './hooks/useFileStore'
import { useJianpuWorker } from './hooks/useJianpuWorker'
import {
  readPartTogglesForFile,
  writePartTogglesForFile,
} from './partToggleCache'
import type { EditorHandle } from './types'
import './App.css'
import './file-tab-bar.css'
import './preview.css'

const shortcutLabel = navigator.platform.startsWith('Mac') ? '⌘↵' : 'Ctrl+↵'

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
  const editorRef = useRef<EditorHandle>(null)
  const skipToggleSaveRef = useRef(false)
  const [currentLine, setCurrentLine] = useState<number | null>(null)
  const {
    parts,
    partsLoading,
    svgs,
    wavUrl,
    audioAvailable,
    pdfAvailable,
    pdfFontsReady,
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
    scoreLineHints,
    notifySelection,
    playSelectedMeasures,
    stopMeasurePlayback,
    highlightedSvgs,
  } = useJianpuWorker(source, disabledParts, disabledLyrics, store.active)

  useEffect(() => {
    skipToggleSaveRef.current = true
    const cached = readPartTogglesForFile(fileId)
    setDisabledParts(new Set(cached?.disabledParts ?? []))
    setDisabledLyrics(new Set(cached?.disabledLyrics ?? []))
  }, [fileId])

  useEffect(() => {
    if (skipToggleSaveRef.current) {
      skipToggleSaveRef.current = false
      return
    }
    writePartTogglesForFile(fileId, {
      disabledParts: [...disabledParts],
      disabledLyrics: [...disabledLyrics],
    })
  }, [fileId, disabledParts, disabledLyrics])

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

  const noPartsSelected =
    parts.length > 0 &&
    parts.every((part) => disabledParts.has(part.abbreviation))

  return (
    <div className="app">
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
                scoreLineHints={scoreLineHints}
                onSelectionChange={notifySelection}
                onCursorLineChange={setCurrentLine}
                onPlayMeasure={
                  measureAudioPlaying
                    ? stopMeasurePlayback
                    : selectedMeasureRange !== null && !measureAudioGenerating
                      ? playSelectedMeasures
                      : undefined
                }
                toolbar={
                  audioAvailable ? (
                    <div
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: '0.5rem',
                      }}
                    >
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
                      <span
                        style={{
                          fontSize: '0.75rem',
                          color: '#888',
                          fontFamily: 'monospace',
                        }}
                      >
                        line {currentLine ?? '?'} |{' '}
                        {selectedMeasureRange !== null
                          ? selectedMeasureRange.start ===
                            selectedMeasureRange.end
                            ? `measure ${selectedMeasureRange.start + 1}`
                            : `measures ${selectedMeasureRange.start + 1}–${selectedMeasureRange.end + 1}`
                          : 'measure null'}
                      </span>
                    </div>
                  ) : null
                }
              />
            </div>
          </div>
        </section>
        <div className="pane-divider" aria-hidden="true" />
        <section className="pane pane--preview">
          <Preview
            svgs={svgs}
            highlightedSvgs={highlightedSvgs}
            rendering={rendering}
            audioGenerating={audioGenerating}
            wavUrl={wavUrl}
            audioAvailable={audioAvailable}
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
                onPartToggle={handlePartToggle}
                onLyricsToggle={handleLyricsToggle}
                loading={partsLoading}
              />
            }
          />
        </section>
      </main>
    </div>
  )
}
