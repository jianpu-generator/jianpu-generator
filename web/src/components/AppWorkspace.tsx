import type { NoteTimingOut, SvgDocumentOut } from 'jianpu-wasm'
import { AlignLeft } from 'lucide-react'
import type { RefObject } from 'react'
import { useEffect, useState } from 'react'
import { MOBILE_BREAKPOINT_QUERY, useMediaQuery } from '../hooks/useMediaQuery'
import type {
  Diagnostic,
  DiagnosticViewZone,
  EditorHandle,
  LyricSpan,
  MeasureSpan,
  NoteSpan,
  PartDeclaration,
  PartInfo,
  PartMode,
  SoundfontValue,
} from '../types'
import type {
  MetadataFieldKey,
  ParsedMetadataFields,
} from '../utils/metadataSource'
import { EditMetadataModal } from './EditMetadataModal'
import { Editor } from './Editor'
import { EditPartsModal } from './EditPartsModal'
import type { LyricCell, NoteCell } from './Preview'
import { Preview } from './Preview'

interface MeasureRange {
  start: number
  end: number
  /** Which measure the preview should scroll to for this selection, when
   * it differs from `start` — see `Preview`'s matching prop doc comment. */
  revealMeasureIndex: number
  /** The exact disjoint measure ranges to highlight in the SVG preview —
   * see `Preview`'s matching prop doc comment. */
  highlightRanges?: { start: number; end: number }[]
}

interface AppWorkspaceProps {
  editorCollapsed: boolean
  setEditorCollapsed: (updater: (collapsed: boolean) => boolean) => void
  /** True while viewing a `#share=` or `#live=` read-only preview — hides
   * the `Editor` entirely and the pane-divider toggle, since there is
   * nothing to edit or expand back into. */
  hideEditor: boolean
  editorRef: RefObject<EditorHandle | null>
  fileId: string
  source: string
  handleSourceChange: (value: string) => void
  /** "Format" toolbar action: drops redundant `# score` lines and
   * normalizes whitespace. */
  handleFormatScore: () => void
  readOnly: boolean
  diagnostics: Diagnostic[]
  diagnosticViewZones: DiagnosticViewZone[]
  measureSpans: MeasureSpan[]
  setSelectedLineRange: (
    range: { firstLine: number; lastLine: number } | null,
  ) => void
  notifySelection: (
    startLine: number,
    endLine: number,
    isEmpty: boolean,
    revealLine?: number,
    measureRanges?: { start: number; end: number }[],
  ) => void
  setEditPartsOpen: (open: boolean) => void
  setEditMetadataOpen: (open: boolean) => void
  forceSave: () => void
  measureAudioPlaying: boolean
  stopMeasurePlayback: () => void
  selectedMeasureRange: MeasureRange | null
  measureAudioGenerating: boolean
  soundfontReady: boolean
  playSelectedMeasures: () => void
  /** True while a note drag-select (see `useNoteSelection`) is active; when
   * set, the editor's Cmd/Ctrl+Enter shortcut plays the selected notes
   * instead of the measure(s) under the cursor. */
  notePlaybackSelectionActive: boolean
  playNoteSelection: () => void
  editPartsOpen: boolean
  partDeclarations: PartDeclaration[]
  parts: PartInfo[]
  handlePartDeclarationChange: (
    abbreviation: string,
    mode: PartMode,
    followTarget: string | null,
    soundfont: SoundfontValue | null,
    volume: number | null,
    octaveOffset: number | null,
  ) => void
  handleShiftPartOctave: (abbreviation: string, delta: number) => void
  previewInstrument: (programNumber: number) => void
  previewPercussion: (key: number) => void
  stopPreviewInstrument: () => void
  previewAudioPlaying: boolean
  editMetadataOpen: boolean
  parsedMetadata: ParsedMetadataFields
  handleMetadataFieldChange: (
    key: MetadataFieldKey,
    value: string | null,
  ) => void
  documents: SvgDocumentOut[]
  highlightedDocuments: SvgDocumentOut[]
  rendering: boolean
  handleSectionJump: (label: string) => void
  handleNoteRangeSelect: (selectedCells: NoteCell[]) => void
  /** Keeps the preview's note highlight in sync with the editor's own
   * current selection (see `useNoteSelection`'s
   * `handleEditorSelectionChange`), the reverse direction of
   * `handleNoteRangeSelect`. */
  handleEditorSelectionChange: (startByte: number, endByte: number) => void
  selectedNoteCells: NoteCell[]
  /** Per-note/rest `(source_part_index, note_id) → measure_index` mapping,
   * used to resolve a measure click/drag into every note cell it contains
   * (see `Preview.tsx`'s `noteCellsInMeasureRange`) without relying on
   * pixel geometry. */
  noteSpans: NoteSpan[]
  /** Fired on mouseup after a lyric-syllable drag-select (see
   * `useLyricSelection`). Independent of `handleNoteRangeSelect` — a lyric
   * drag never selects/highlights notes and vice versa. */
  handleLyricRangeSelect: (selectedCells: LyricCell[]) => void
  /** Keeps the preview's lyric highlight in sync with the editor's own
   * current selection, the reverse direction of `handleLyricRangeSelect`
   * (mirrors `handleEditorSelectionChange`, kept fully separate). */
  handleLyricEditorSelectionChange: (startByte: number, endByte: number) => void
  selectedLyricCells: LyricCell[]
  /** Per-lyric-syllable `(source_part_index, note_id, verse) → measure_index`
   * mapping, used to resolve a measure click/drag into every lyric cell it
   * contains alongside `noteSpans` (see `Preview.tsx`'s
   * `lyricCellsInMeasureRange`). */
  lyricSpans: LyricSpan[]
  /** Fired for a measure/bar-line click or drag with both the note cells and
   * lyric cells it resolved — see `Preview.tsx`'s `onMeasureRangeSelect`. */
  handleMeasureRangeSelect: (
    noteCells: NoteCell[],
    lyricCells: LyricCell[],
  ) => void
  audioGenerating: boolean
  wavUrl: string | null
  wavFilename: string
  mp3Exporting: boolean
  mp3Url: string | null
  mp3Filename: string
  noteTimings: NoteTimingOut[]
  measureAudioNoteTimings: NoteTimingOut[]
  measureAudioElement: HTMLAudioElement | null
  noPartsSelected: boolean
}

export function AppWorkspace({
  editorCollapsed,
  setEditorCollapsed,
  hideEditor,
  editorRef,
  fileId,
  source,
  handleSourceChange,
  handleFormatScore,
  readOnly,
  diagnostics,
  diagnosticViewZones,
  measureSpans,
  setSelectedLineRange,
  notifySelection,
  setEditPartsOpen,
  setEditMetadataOpen,
  forceSave,
  measureAudioPlaying,
  stopMeasurePlayback,
  selectedMeasureRange,
  measureAudioGenerating,
  soundfontReady,
  playSelectedMeasures,
  notePlaybackSelectionActive,
  playNoteSelection,
  editPartsOpen,
  partDeclarations,
  parts,
  handlePartDeclarationChange,
  handleShiftPartOctave,
  previewInstrument,
  previewPercussion,
  stopPreviewInstrument,
  previewAudioPlaying,
  editMetadataOpen,
  parsedMetadata,
  handleMetadataFieldChange,
  documents,
  highlightedDocuments,
  rendering,
  handleSectionJump,
  handleNoteRangeSelect,
  handleEditorSelectionChange,
  selectedNoteCells,
  noteSpans,
  handleLyricRangeSelect,
  handleLyricEditorSelectionChange,
  selectedLyricCells,
  lyricSpans,
  handleMeasureRangeSelect,
  audioGenerating,
  wavUrl,
  wavFilename,
  mp3Exporting,
  mp3Url,
  mp3Filename,
  noteTimings,
  measureAudioNoteTimings,
  measureAudioElement,
  noPartsSelected,
}: AppWorkspaceProps) {
  const [editorPaneEl, setEditorPaneEl] = useState<HTMLDivElement | null>(null)
  const isMobile = useMediaQuery(MOBILE_BREAKPOINT_QUERY)
  // Below the mobile breakpoint only one pane is visible at a time, so
  // showing the editor must collapse the preview instead of sitting beside it.
  const previewCollapsed = isMobile && !hideEditor && !editorCollapsed
  // On mobile the divider sits between a stacked editor (above) and preview
  // (below), so the chevron points up/down instead of left/right.
  const toggleIconRotationDeg = isMobile
    ? editorCollapsed
      ? -90
      : 90
    : editorCollapsed
      ? 180
      : 0

  // Default to showing the preview on mobile, where only one pane fits.
  useEffect(() => {
    if (window.matchMedia(MOBILE_BREAKPOINT_QUERY).matches) {
      setEditorCollapsed(() => true)
    }
  }, [setEditorCollapsed])

  return (
    <main className="workspace">
      <section
        className={[
          'pane',
          'pane--editor',
          'pane--collapsible',
          editorCollapsed ? 'pane--editor-collapsed' : '',
        ]
          .filter(Boolean)
          .join(' ')}
      >
        <div className="editor-layout">
          <div className="editor-main" ref={setEditorPaneEl}>
            {hideEditor ? null : (
              <Editor
                ref={editorRef}
                path={fileId}
                value={source}
                onChange={handleSourceChange}
                readOnly={readOnly}
                diagnostics={diagnostics}
                diagnosticViewZones={diagnosticViewZones}
                measureSpans={measureSpans}
                onSelectionChange={(firstLine, lastLine, isEmpty) => {
                  setSelectedLineRange(null)
                  notifySelection(firstLine, lastLine, isEmpty)
                }}
                onSelectionOffsetChange={(startOffset, endOffset) => {
                  handleEditorSelectionChange(startOffset, endOffset)
                  handleLyricEditorSelectionChange(startOffset, endOffset)
                }}
                onEditPartsClick={() => setEditPartsOpen(true)}
                onEditMetadataClick={() => setEditMetadataOpen(true)}
                onForceSave={forceSave}
                onPlayMeasure={
                  measureAudioPlaying
                    ? stopMeasurePlayback
                    : !measureAudioGenerating && soundfontReady
                      ? notePlaybackSelectionActive
                        ? playNoteSelection
                        : selectedMeasureRange !== null
                          ? playSelectedMeasures
                          : undefined
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
              onShiftPartOctave={handleShiftPartOctave}
              previewInstrument={previewInstrument}
              previewPercussion={previewPercussion}
              stopPreviewInstrument={stopPreviewInstrument}
              previewAudioPlaying={previewAudioPlaying}
            />
            <EditMetadataModal
              open={editMetadataOpen}
              onOpenChange={setEditMetadataOpen}
              metadata={parsedMetadata}
              onFieldChange={handleMetadataFieldChange}
              container={editorPaneEl}
            />
          </div>
        </div>
      </section>
      <div className="pane-divider">
        {hideEditor ? null : (
          <div className="pane-divider-toggles">
            <button
              type="button"
              className="pane-divider-format-toggle"
              onClick={handleFormatScore}
              title="Format score"
              aria-label="Format score"
            >
              <AlignLeft size={12} aria-hidden="true" />
            </button>
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
                  transform: `rotate(${toggleIconRotationDeg}deg)`,
                }}
                aria-hidden="true"
              >
                ‹
              </span>
            </button>
          </div>
        )}
      </div>
      <section
        className={[
          'pane',
          'pane--preview',
          'pane--collapsible',
          previewCollapsed ? 'pane--preview-collapsed' : '',
        ]
          .filter(Boolean)
          .join(' ')}
      >
        <Preview
          documents={documents}
          highlightedDocuments={highlightedDocuments}
          rendering={rendering}
          onSectionLabelClick={handleSectionJump}
          onNoteRangeSelect={handleNoteRangeSelect}
          selectedNoteCells={selectedNoteCells}
          noteSpans={noteSpans}
          onLyricRangeSelect={handleLyricRangeSelect}
          selectedLyricCells={selectedLyricCells}
          lyricSpans={lyricSpans}
          onMeasureRangeSelect={handleMeasureRangeSelect}
          selectedMeasureRange={selectedMeasureRange}
          audioGenerating={audioGenerating}
          wavUrl={wavUrl}
          wavFilename={wavFilename}
          mp3Exporting={mp3Exporting}
          mp3Url={mp3Url}
          mp3Filename={mp3Filename}
          noteTimings={noteTimings}
          measureAudioNoteTimings={measureAudioNoteTimings}
          measureAudioElement={measureAudioElement}
          emptyMessage={
            noPartsSelected ? 'No parts selected.' : 'No preview yet.'
          }
        />
      </section>
    </main>
  )
}
