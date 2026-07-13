import type { SvgDocumentOut } from 'jianpu-wasm'
import type { RefObject } from 'react'
import type { SharePayload } from '../shareUrl'
import type {
  Diagnostic,
  DiagnosticViewZone,
  EditorHandle,
  MeasureSpan,
  PartDeclaration,
  PartInfo,
  PartMode,
  SoundfontValue,
} from '../types'
import type { MetadataKey, ParsedMetadataFields } from '../utils/metadataSource'
import { EditMetadataModal } from './EditMetadataModal'
import { Editor } from './Editor'
import { EditPartsModal } from './EditPartsModal'
import { PartToggles } from './PartToggles'
import { Preview } from './Preview'

interface MeasureRange {
  start: number
  end: number
}

interface AppWorkspaceProps {
  editorCollapsed: boolean
  setEditorCollapsed: (updater: (collapsed: boolean) => boolean) => void
  sharedPreview: SharePayload | null
  editorRef: RefObject<EditorHandle | null>
  fileId: string
  source: string
  handleSourceChange: (value: string) => void
  readOnly: boolean
  diagnostics: Diagnostic[]
  diagnosticViewZones: DiagnosticViewZone[]
  measureSpans: MeasureSpan[]
  setSelectedLineRange: (
    range: { firstLine: number; lastLine: number } | null,
  ) => void
  notifySelection: (startLine: number, endLine: number) => void
  setEditPartsOpen: (open: boolean) => void
  setEditMetadataOpen: (open: boolean) => void
  forceSave: () => void
  measureAudioPlaying: boolean
  stopMeasurePlayback: () => void
  selectedMeasureRange: MeasureRange | null
  measureAudioGenerating: boolean
  soundfontReady: boolean
  playSelectedMeasures: () => void
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
  handleMeasureRangeSelect: (start: number, end: number) => void
  handleSectionJump: (label: string) => void
  audioGenerating: boolean
  wavUrl: string | null
  wavFilename: string
  measureTimes: number[]
  measureAudioTimes: number[]
  measureAudioElement: HTMLAudioElement | null
  noPartsSelected: boolean
  disabledParts: ReadonlySet<string>
  disabledLyrics: ReadonlySet<string>
  soloedParts: ReadonlySet<string>
  handlePartToggle: (abbreviation: string, enabled: boolean) => void
  handleLyricsToggle: (abbreviation: string, enabled: boolean) => void
  handleSoloToggle: (abbreviation: string, soloed: boolean) => void
}

export function AppWorkspace({
  editorCollapsed,
  setEditorCollapsed,
  sharedPreview,
  editorRef,
  fileId,
  source,
  handleSourceChange,
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
  handleMeasureRangeSelect,
  handleSectionJump,
  audioGenerating,
  wavUrl,
  wavFilename,
  measureTimes,
  measureAudioTimes,
  measureAudioElement,
  noPartsSelected,
  disabledParts,
  disabledLyrics,
  soloedParts,
  handlePartToggle,
  handleLyricsToggle,
  handleSoloToggle,
}: AppWorkspaceProps) {
  return (
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
              previewPercussion={previewPercussion}
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
  )
}
