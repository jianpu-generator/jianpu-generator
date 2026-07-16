import { useEffect, useState } from 'react'
import type { FileStoreState } from '../fileStore'
import { isReadOnlyFile, sortedBinNames } from '../fileStore'
import type { DisplaySaveStatus } from '../hooks/useStorageBackend'
import { BinMenu } from './BinMenu'
import { DemoFileSwitcher } from './DemoFileSwitcher'
import { ExportControls } from './ExportControls'
import { FileSwitcher } from './FileSwitcher'
import { PlayFromCurrentMeasureButton } from './PlayFromCurrentMeasureButton'
import { PlayMeasureButton } from './PlayMeasureButton'

interface MeasureRange {
  start: number
  end: number
}

interface AppHeaderProps {
  audioAvailable?: boolean
  selectedMeasureRange: MeasureRange | null
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
  onRestore: (name: string) => void
  restoringFileName?: string | null
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
}

export function AppHeader({
  audioAvailable,
  selectedMeasureRange,
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
  onRestore,
  restoringFileName,
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
}: AppHeaderProps) {
  // The "My Files" trigger keeps showing the last user file that was active
  // even while a (separately-dropdown'd) demo file is currently open, so
  // switching to a demo file doesn't make the "My Files" trigger look empty.
  // Falls back to a placeholder once that file no longer exists (e.g. it was
  // deleted) or no user file has ever been active yet.
  const [lastActiveUserFileName, setLastActiveUserFileName] = useState<
    string | null
  >(() => (isReadOnlyFile(store.active) ? null : store.active))
  useEffect(() => {
    if (!isReadOnlyFile(store.active)) setLastActiveUserFileName(store.active)
  }, [store.active])
  const triggerLabel =
    lastActiveUserFileName !== null &&
    store.userFiles[lastActiveUserFileName] !== undefined
      ? lastActiveUserFileName
      : 'Untitled'

  return (
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
          triggerLabel={triggerLabel}
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
        />
        <DemoFileSwitcher active={store.active} onSelect={onSelect} />
        <BinMenu
          binNames={sortedBinNames(store)}
          onRestore={onRestore}
          restoringName={restoringFileName}
        />
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
