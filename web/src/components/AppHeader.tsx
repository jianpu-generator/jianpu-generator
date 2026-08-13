import type { FileStoreState } from '../fileStore'
import { sortedBinNames } from '../fileStore'
import type { LiveViewerStatus } from '../hooks/useLiveViewer'
import type { DisplaySaveStatus } from '../hooks/useStorageBackend'
import type { SharePayload } from '../shareUrl'
import { ExportControls } from './ExportControls'
import { FileSwitcher } from './FileSwitcher'
import { GoLiveButton } from './GoLiveButton'
import { LiveShareBanner } from './LiveShareBanner'
import { PlayFromCurrentMeasureButton } from './PlayFromCurrentMeasureButton'
import { PlayMeasureButton } from './PlayMeasureButton'
import { SharedPreviewBanner } from './SharedPreviewBanner'

interface MeasureRange {
  start: number
  end: number
}

interface LiveShareHeaderProps {
  sharedPreview: SharePayload | null
  onImportShared: () => void
  onDismissShared: () => void
  /** Non-null while a `#live=` link is being viewed. Takes a back seat to
   * `sharedPreview` if both are somehow present at once (documented edge
   * case in the Live Share plan). */
  viewerActive: boolean
  viewerStatus: LiveViewerStatus
  viewerFilename: string | null
  onImportLive: () => void
  isLive: boolean
  liveUrl: string | null
  onStartLive: () => string
  onStopLive: () => void
}

interface AppHeaderProps {
  audioAvailable?: boolean
  selectedMeasureRange: MeasureRange | null
  selectedSequenceRange: MeasureRange | null
  measureAudioGenerating: boolean
  soundfontReady: boolean
  measureAudioPlaying: boolean
  playSelectedMeasures: () => void
  playFromCurrentMeasure: () => void
  /** True while a note drag-select (see `useNoteSelection`) is active; when
   * set, `PlayMeasureButton` plays only the selected parts, muted elsewhere,
   * over the selection's measure range instead of the measure(s) under the
   * cursor. */
  notePlaybackSelectionActive: boolean
  playNoteSelection: () => void
  stopMeasurePlayback: () => void
  shortcutLabel: string
  playFromCurrentMeasureShortcutLabel: string
  store: FileStoreState
  onSelect: (name: string) => void
  onCreate: () => void
  onDuplicate: () => void
  onRename: (from: string, to: string) => void
  onDelete: (name: string) => void
  onOpenStorageSettings: () => void
  saveStatus: DisplaySaveStatus
  autosaveDeadline: number | null
  creatingFile?: boolean
  deletingFileName?: string | null
  duplicatingFile?: boolean
  renamingFileName?: string | null
  isLoadingGithub?: boolean
  onOpenBin: () => void
  hasDocuments: boolean
  rendering: boolean
  audioGenerating?: boolean
  wavUrl?: string | null
  onGenerateAudio?: () => void
  pdfAvailable?: boolean
  pdfFontsReady?: boolean
  pdfExporting?: boolean
  onExportPdf?: () => void
  splitPdfExporting?: boolean
  onExportSplitPdf?: () => void
  midiAvailable?: boolean
  midiExporting?: boolean
  onExportMidi?: () => void
  splitMidiExporting?: boolean
  onExportSplitMidi?: () => void
  splitWavExporting?: boolean
  onExportSplitWav?: () => void
  partsCount?: number
  importing?: boolean
  onImportFile?: (file: File) => void
  liveShare: LiveShareHeaderProps
}

export function AppHeader({
  audioAvailable,
  selectedMeasureRange,
  selectedSequenceRange,
  measureAudioGenerating,
  soundfontReady,
  measureAudioPlaying,
  playSelectedMeasures,
  playFromCurrentMeasure,
  notePlaybackSelectionActive,
  playNoteSelection,
  stopMeasurePlayback,
  shortcutLabel,
  playFromCurrentMeasureShortcutLabel,
  store,
  onSelect,
  onCreate,
  onDuplicate,
  onRename,
  onDelete,
  onOpenStorageSettings,
  saveStatus,
  autosaveDeadline,
  creatingFile,
  deletingFileName,
  duplicatingFile,
  renamingFileName,
  isLoadingGithub,
  onOpenBin,
  hasDocuments,
  rendering,
  audioGenerating,
  wavUrl,
  onGenerateAudio,
  pdfAvailable,
  pdfFontsReady,
  pdfExporting,
  onExportPdf,
  splitPdfExporting,
  onExportSplitPdf,
  midiAvailable,
  midiExporting,
  onExportMidi,
  splitMidiExporting,
  onExportSplitMidi,
  splitWavExporting,
  onExportSplitWav,
  partsCount,
  importing,
  onImportFile,
  liveShare,
}: AppHeaderProps) {
  const { sharedPreview, viewerActive: liveViewerActive } = liveShare
  return (
    <header className="app-header">
      <h1>簡譜</h1>
      {sharedPreview ? (
        <SharedPreviewBanner
          filename={sharedPreview.filename}
          onImport={liveShare.onImportShared}
          onDiscard={liveShare.onDismissShared}
        />
      ) : (
        liveViewerActive && (
          <LiveShareBanner
            status={liveShare.viewerStatus}
            filename={liveShare.viewerFilename}
            onImport={liveShare.onImportLive}
          />
        )
      )}
      {audioAvailable && (
        <PlayMeasureButton
          disabled={
            (notePlaybackSelectionActive
              ? false
              : selectedMeasureRange === null) ||
            measureAudioGenerating ||
            !soundfontReady
          }
          loading={measureAudioGenerating}
          playing={measureAudioPlaying}
          measureRange={selectedMeasureRange}
          noteSelectionActive={notePlaybackSelectionActive}
          onClick={
            notePlaybackSelectionActive
              ? playNoteSelection
              : playSelectedMeasures
          }
          onPause={stopMeasurePlayback}
          shortcutLabel={shortcutLabel}
        />
      )}
      {audioAvailable && (
        <PlayFromCurrentMeasureButton
          disabled={
            selectedSequenceRange === null ||
            measureAudioGenerating ||
            !soundfontReady
          }
          loading={measureAudioGenerating}
          playing={measureAudioPlaying}
          currentMeasure={selectedSequenceRange?.start ?? null}
          onClick={playFromCurrentMeasure}
          onPause={stopMeasurePlayback}
          shortcutLabel={playFromCurrentMeasureShortcutLabel}
        />
      )}
      <div className="app-header-actions">
        {!sharedPreview && !liveViewerActive && (
          <FileSwitcher
            store={store}
            triggerLabel={store.active}
            onSelect={onSelect}
            onCreate={onCreate}
            onDuplicate={onDuplicate}
            onRename={onRename}
            onDelete={onDelete}
            onOpenStorageSettings={onOpenStorageSettings}
            saveStatus={saveStatus}
            autosaveDeadline={autosaveDeadline}
            creating={creatingFile}
            deletingName={deletingFileName}
            duplicating={duplicatingFile}
            renamingName={renamingFileName}
            isLoadingGithub={isLoadingGithub}
            importing={importing}
            onImportFile={onImportFile}
            binNames={sortedBinNames(store)}
            onOpenBin={onOpenBin}
          />
        )}
        {!sharedPreview && !liveViewerActive && (
          <GoLiveButton
            isLive={liveShare.isLive}
            liveUrl={liveShare.liveUrl}
            onStartLive={liveShare.onStartLive}
            onStopLive={liveShare.onStopLive}
          />
        )}
        <ExportControls
          hasDocuments={hasDocuments}
          rendering={rendering}
          audioGenerating={audioGenerating}
          wavUrl={wavUrl}
          soundfontReady={soundfontReady}
          onGenerateAudio={onGenerateAudio}
          pdfAvailable={pdfAvailable}
          pdfFontsReady={pdfFontsReady}
          pdfExporting={pdfExporting}
          onExportPdf={onExportPdf}
          splitPdfExporting={splitPdfExporting}
          onExportSplitPdf={onExportSplitPdf}
          midiAvailable={midiAvailable}
          midiExporting={midiExporting}
          onExportMidi={onExportMidi}
          splitMidiExporting={splitMidiExporting}
          onExportSplitMidi={onExportSplitMidi}
          audioAvailable={audioAvailable}
          splitWavExporting={splitWavExporting}
          onExportSplitWav={onExportSplitWav}
          partsCount={partsCount}
          isLoadingGithub={isLoadingGithub}
        />
      </div>
    </header>
  )
}
