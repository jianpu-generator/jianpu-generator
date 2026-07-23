import type { FileStoreState } from '../fileStore'
import { sortedBinNames } from '../fileStore'
import type { DisplaySaveStatus } from '../hooks/useStorageBackend'
import type { SharePayload } from '../shareUrl'
import { ExportControls } from './ExportControls'
import { FileSwitcher } from './FileSwitcher'
import { PlayFromCurrentMeasureButton } from './PlayFromCurrentMeasureButton'
import { PlayMeasureButton } from './PlayMeasureButton'
import { SharedPreviewBanner } from './SharedPreviewBanner'

interface MeasureRange {
  start: number
  end: number
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
  sharedPreview: SharePayload | null
  onImportShared: () => void
  onDismissShared: () => void
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
  sharedPreview,
  onImportShared,
  onDismissShared,
}: AppHeaderProps) {
  return (
    <header className="app-header">
      <h1>簡譜</h1>
      {sharedPreview && (
        <SharedPreviewBanner
          filename={sharedPreview.filename}
          onImport={onImportShared}
          onDiscard={onDismissShared}
        />
      )}
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
        {!sharedPreview && (
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
