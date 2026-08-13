import type {
  NoteTimingOut,
  PartMeasureRangesOut,
  SvgDocumentOut,
} from 'jianpu-wasm'
import { merge_unzipped_text } from 'jianpu-wasm'
import { AlignLeft, Columns2 } from 'lucide-react'
import type { RefObject } from 'react'
import { useCallback, useEffect, useState } from 'react'
import { MOBILE_BREAKPOINT_QUERY, useMediaQuery } from '../hooks/useMediaQuery'
import type {
  Diagnostic,
  DiagnosticViewZone,
  EditorHandle,
  MeasureSpan,
  NoteSpan,
  PartDeclaration,
  PartInfo,
  PartMode,
  SoundfontValue,
} from '../types'
import type { MetadataKey, ParsedMetadataFields } from '../utils/metadataSource'
import { EditMetadataModal } from './EditMetadataModal'
import { Editor } from './Editor'
import { EditPartsModal } from './EditPartsModal'
import type { NoteCell } from './Preview'
import { Preview } from './Preview'

interface MeasureRange {
  start: number
  end: number
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
  /** Zipped-view "Format" toolbar action: drops redundant `# score` lines
   * and normalizes whitespace. Active only while Unzipped view is off. */
  handleFormatScore: () => void
  /** Unzipped-view "Format" toolbar action: breaks each measure onto its own
   * line. Active only while Unzipped view is on. */
  handleFormatUnzippedText: () => void
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
  ) => void
  notifyUnzippedSelection: (
    startOffset: number,
    endOffset: number,
    isEmpty: boolean,
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
  previewInstrument: (programNumber: number) => void
  previewPercussion: (key: number) => void
  stopPreviewInstrument: () => void
  previewAudioPlaying: boolean
  editMetadataOpen: boolean
  parsedMetadata: ParsedMetadataFields
  handleMetadataFieldChange: (key: MetadataKey, value: string | null) => void
  documents: SvgDocumentOut[]
  highlightedDocuments: SvgDocumentOut[]
  rendering: boolean
  handleSectionJump: (label: string) => void
  handleNoteRangeSelect: (selectedCells: NoteCell[]) => void
  /** Keeps the preview's note highlight in sync with the editor's own
   * current selection (Zipped view only — see `useNoteSelection`'s
   * `handleEditorSelectionChange`), the reverse direction of
   * `handleNoteRangeSelect`. */
  handleEditorSelectionChange: (startByte: number, endByte: number) => void
  selectedNoteCells: NoteCell[]
  /** Per-note/rest `(source_part_index, note_id) → measure_index` mapping,
   * used to resolve a measure click/drag into every note cell it contains
   * (see `Preview.tsx`'s `noteCellsInMeasureRange`) without relying on
   * pixel geometry. */
  noteSpans: NoteSpan[]
  audioGenerating: boolean
  wavUrl: string | null
  wavFilename: string
  noteTimings: NoteTimingOut[]
  measureAudioNoteTimings: NoteTimingOut[]
  measureAudioElement: HTMLAudioElement | null
  noPartsSelected: boolean
  /** Whether the whole-document Unzipped view (Zipped⇄Unzipped toggle) is
   * currently shown instead of the normal Zipped editor. */
  unzippedView: boolean
  onToggleUnzippedView: () => void
  /** The Unzipped view projection of `source`, kept in sync by the caller
   * whenever `unzippedView` is true. */
  unzippedText: string
  /** Per-part, per-measure byte ranges into `unzippedText`, used to relocate
   * Zipped-source-relative diagnostics onto the Unzipped view's text. */
  partMeasureRanges: PartMeasureRangesOut[]
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
  handleFormatUnzippedText,
  readOnly,
  diagnostics,
  diagnosticViewZones,
  measureSpans,
  setSelectedLineRange,
  notifySelection,
  notifyUnzippedSelection,
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
  audioGenerating,
  wavUrl,
  wavFilename,
  noteTimings,
  measureAudioNoteTimings,
  measureAudioElement,
  noPartsSelected,
  unzippedView,
  onToggleUnzippedView,
  unzippedText,
  partMeasureRanges,
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

  const handleEditorChange = useCallback(
    (value: string) => {
      if (!unzippedView) {
        handleSourceChange(value)
        return
      }
      const result = merge_unzipped_text(source, value)
      if (result.status === 'ok') {
        handleSourceChange(result.text)
      }
    },
    [unzippedView, source, handleSourceChange],
  )

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
                path={unzippedView ? `${fileId}::unzipped` : fileId}
                value={unzippedView ? unzippedText : source}
                onChange={handleEditorChange}
                readOnly={readOnly}
                diagnostics={diagnostics}
                diagnosticViewZones={diagnosticViewZones}
                measureSpans={measureSpans}
                unzippedView={unzippedView}
                partMeasureRanges={partMeasureRanges}
                onSelectionChange={(firstLine, lastLine, isEmpty) => {
                  setSelectedLineRange(null)
                  if (!unzippedView) {
                    notifySelection(firstLine, lastLine, isEmpty)
                  }
                }}
                onSelectionOffsetChange={(startOffset, endOffset, isEmpty) => {
                  if (unzippedView) {
                    notifyUnzippedSelection(startOffset, endOffset, isEmpty)
                  } else {
                    handleEditorSelectionChange(startOffset, endOffset)
                  }
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
              className={[
                'pane-divider-view-toggle',
                unzippedView ? 'pane-divider-view-toggle--active' : '',
              ]
                .filter(Boolean)
                .join(' ')}
              onClick={onToggleUnzippedView}
              title={unzippedView ? 'Show Zipped view' : 'Show Unzipped view'}
              aria-label={
                unzippedView ? 'Show Zipped view' : 'Show Unzipped view'
              }
              aria-pressed={unzippedView}
            >
              <Columns2 size={12} aria-hidden="true" />
            </button>
            <button
              type="button"
              className="pane-divider-format-toggle"
              onClick={
                unzippedView ? handleFormatUnzippedText : handleFormatScore
              }
              title={unzippedView ? 'Format unzipped text' : 'Format score'}
              aria-label={
                unzippedView ? 'Format unzipped text' : 'Format score'
              }
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
          audioGenerating={audioGenerating}
          wavUrl={wavUrl}
          wavFilename={wavFilename}
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
